use std::str::FromStr;

use actix_web::{
    web::{Form, Header},
    Error, HttpRequest, HttpResponse, Result,
};

use bson::{doc, oid::ObjectId};

use serde_json::json;
use tokio::fs::{self};

use crate::{
    structs::{
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
    let token = User::generate_token();

    let user = User::from(&data.username, &data.password, &data.email, &token.clone());

    let result = users.insert_one(&user, None).await;

    fs::create_dir(format!(
        "{}/{}",
        state.config.storage.path,
        user._id.to_hex()
    ))
    .await?;

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

    // Get the authorization header
    let auth_token = headers.authorization.clone().unwrap();

    // Get the database
    let state = request.app_data::<AppState>().unwrap();
    let users = state.database.collection::<User>("users");

    // Find the user with the specified token
    let requester = users
        .find_one(doc! {"token": auth_token}, None)
        .await
        .unwrap();

    // If the user is not found, return unauthorized
    if requester.is_none() {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    }

    // Get the requester
    let requester = requester.unwrap();

    // Get the desired user
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
        let db_result = users.delete_one(doc! {"_id": user._id}, None).await;

        if db_result.is_err() {
            return Ok(HttpResponse::InternalServerError()
                .body("There was an error deleting the user from the database"));
        }

        let fs_result = fs::remove_dir_all(format!(
            "{}/{}",
            state.config.storage.path,
            user._id.to_hex()
        ))
        .await;

        if fs_result.is_err() {
            return Ok(HttpResponse::InternalServerError()
                .body("There was an error deleting the user's storage directory"));
        }

        return Ok(HttpResponse::NoContent().body("Deleted"));
    }

    // If the user is not the same user as the requester, check if the requester has admin privileges
    if !requester.privileges.contains(Privileges::ADMIN) {
        return Ok(HttpResponse::Forbidden().body("Forbidden"));
    }

    let db_result = users.delete_one(doc! {"_id": user._id}, None).await;

    if db_result.is_err() {
        return Ok(HttpResponse::InternalServerError()
            .body("There was an error deleting the user from the database"));
    }

    let fs_result = fs::remove_dir_all(format!(
        "{}/{}",
        state.config.storage.path,
        user._id.to_hex()
    ))
    .await;

    if fs_result.is_err() {
        return Ok(HttpResponse::InternalServerError()
            .body("There was an error deleting the user's storage directory"));
    }

    Ok(HttpResponse::NoContent().body("Deleted"))
}
