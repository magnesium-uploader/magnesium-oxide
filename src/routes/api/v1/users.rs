use std::str::FromStr;

use actix_web::{
    web::{Form, Header},
    Error, HttpRequest, HttpResponse, Result,
};

use argon2::{
    password_hash::PasswordVerifier
};

use bson::{doc, oid::ObjectId};

use serde_json::json;

use crate::{
    modules::{storage::Storage, crypto::verify_syn_argon2},
    structs::{
        files::File,
        users::{User, UserCreateRequest, UserIdRequest},
        AuthorizationHeader, Privileges,
    },
    AppState,
};

/// This endpoint creates a new user in the database and initializes their storage,
/// the user is returned with a token that can be used for authorization in subsequent requests.
pub async fn create_user(
    request: HttpRequest,
    data: Form<UserCreateRequest>,
) -> Result<HttpResponse, Error> {
    let state = request.app_data::<AppState>().unwrap();
    let users = state.database.collection::<User>("users");
    let storage = state.storage.clone();

    match verify_syn_argon2(&data.p_hash) {
        true => {}
        false => {
            return Ok(HttpResponse::BadRequest().body("p-hash is not valid"))
        }
    }

    let token = User::generate_token();
    
    let user = User::from(&data.username, &data.p_hash, &data.email, &token.clone());

    let result = users.insert_one(&user, None).await;

    match storage {
        Storage::Local(ref storage) => {
            let path = format!("{}/{}", storage, user._id.to_hex());
            tokio::fs::create_dir_all(&path).await?;
        }
        Storage::S3(ref _storage) => {
            todo!("S3 storage");
        }
    }

    if result.is_err() {
        return Ok(HttpResponse::InternalServerError().body("Internal Server Error"));
    }

    Ok(HttpResponse::Created().json(json!({ "token": token })))
}

/// This endpoint returns a user's information if the requestee has administrative privileges.
pub async fn get_user(
    request: HttpRequest,
    data: Form<UserIdRequest>,
    headers: &Header<AuthorizationHeader>,
) -> Result<HttpResponse, Error> {
    if headers.authorization.is_none() {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    }

    let state = request.app_data::<AppState>().unwrap();
    let users = state.database.collection::<User>("users");

    let auth_token = headers.authorization.clone().unwrap();

    let requester = users
        .find_one(doc! {"token": auth_token}, None)
        .await
        .unwrap();

    if requester.is_none() {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    }

    let requester = requester.unwrap();

    if !requester.privileges.contains(Privileges::ADMIN) {
        return Ok(HttpResponse::Forbidden().body("Forbidden"));
    }

    let _id = match ObjectId::from_str(&data.id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().body("The specfiied id is not valid"));
        }
    };

    let user = users.find_one(doc! { "_id": _id }, None).await.unwrap();

    if user.is_none() {
        return Ok(HttpResponse::NotFound().body("Not Found"));
    }

    Ok(HttpResponse::Ok().json(user.unwrap()))
}

/// This endpoint deletes a user from the database if one of the following conditions are met:
/// 1. The requestee has administrative privileges.
/// 2. The requestee is deleting their own account.
///
/// **Note:** The user must specify a valid ObjectId and authorize with a valid token.
pub async fn delete_user(
    request: HttpRequest,
    data: Form<UserIdRequest>,
    headers: &Header<AuthorizationHeader>,
) -> Result<HttpResponse, Error> {
    // If there is no authorization header, return unauthorized
    if headers.authorization.is_none() {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    }

    let auth_token = headers.authorization.clone().unwrap();

    let state = request.app_data::<AppState>().unwrap();
    let users = state.database.collection::<User>("users");
    let files = state.database.collection::<File>("files");
    let storage = state.storage.clone();

    let requester = users
        .find_one(doc! {"token": auth_token}, None)
        .await
        .unwrap();

    if requester.is_none() {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    }

    let requester = requester.unwrap();

    let user = users
        .find_one(doc! {"_id": ObjectId::from_str(&data.id).unwrap()}, None)
        .await
        .unwrap();

    if user.is_none() {
        return Ok(HttpResponse::NotFound().body("Not Found"));
    }

    let user = user.unwrap();

    // check if the user to be deleted is the same user as the requester
    // if so, the user is deleting themselves, so no need to check for privileges
    if requester._id.to_hex() == user._id.to_hex() {
        let user_result = users.delete_one(doc! {"_id": user._id}, None).await;

        if user_result.is_err() {
            return Ok(HttpResponse::InternalServerError()
                .body("There was an error deleting the user from the database"));
        }

        let files_result = files.delete_many(doc! {"uploader": user._id}, None).await;

        if files_result.is_err() {
            return Ok(HttpResponse::InternalServerError()
                .body("There was an error deleting the user's files from the database"));
        }

        match storage {
            Storage::Local(ref storage) => {
                let path = format!("{}/{}", storage, user._id.to_hex());
                match tokio::fs::remove_dir_all(&path).await {
                    Ok(_) => {
                        return Ok(HttpResponse::Ok().body("User deleted"));
                    }
                    Err(_) => {
                        return Ok(HttpResponse::InternalServerError()
                            .body("There was an error deleting the user's storage"));
                    }
                }
            }
            Storage::S3(ref _storage) => {
                todo!("S3 storage");
            }
        };
    };

    // If the user is not the same user as the requester, check if the requester has admin privileges
    if !requester.privileges.contains(Privileges::ADMIN) {
        return Ok(HttpResponse::Forbidden().body("Forbidden"));
    }

    let user_result = users.delete_one(doc! {"_id": user._id}, None).await;

    if user_result.is_err() {
        return Ok(HttpResponse::InternalServerError()
            .body("There was an error deleting the user from the database"));
    }

    let files_result = files.delete_many(doc! {"uploader": user._id}, None).await;

    if files_result.is_err() {
        return Ok(HttpResponse::InternalServerError()
            .body("There was an error deleting the user's files from the database"));
    }

    match storage {
        Storage::Local(ref storage) => {
            let path = format!("{}/{}", storage, user._id.to_hex());
            match tokio::fs::remove_dir_all(&path).await {
                Ok(_) => {
                    return Ok(HttpResponse::Ok().body("User deleted"));
                }
                Err(_) => {
                    return Ok(HttpResponse::InternalServerError()
                        .body("There was an error deleting the user's storage"));
                }
            }
        }
        Storage::S3(ref _storage) => {
            todo!("S3 storage");
        }
    };
}
