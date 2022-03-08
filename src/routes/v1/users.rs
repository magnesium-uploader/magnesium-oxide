use actix_web::{HttpResponse, Responder, Result, web};
use mongodb::{
    bson::{doc, Document},
    Database,
};
use regex::Regex;
use sha3::{Digest, Sha3_512};

use crate::AppState;
use crate::log;
use crate::routes::v1::types::{DeleteRequest, MessageResponse, Privileges, UserRequest, UserResponse};

use super::utils::check_privilege;

pub fn sanatize(s: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();
    return re.replace_all(s, "").trim().to_ascii_lowercase();
}

/// POST: /api/v1/users
/// 
/// Create a new user
/// # Returns:
///  * `HttpResponse::Created()` - User created
///  * `HttpResponse::Confict()` - User already exists
/// # Parameters (JSON body):
///  * `username` - String
///  * `password` - String
pub async fn create(
    data: web::Data<AppState>,
    request: web::Json<UserRequest>,
) -> Result<impl Responder> {
    log::debug("POST: /api/v1/users");
    let database: Database = data.database.clone();
    let userscollection = database.collection::<Document>("users");

    if userscollection
        .find_one(doc! {"username": &request.username}, None)
        .await
        .unwrap()
        != None
    {
        return Ok(HttpResponse::Conflict().json(MessageResponse {
            message: "User already exists".to_string(),
        }));
    }

    let api_key = uuid::Uuid::new_v4().to_string();

    let hashed_password = {
        let mut hasher = Sha3_512::new();
        hasher.update(request.password.as_bytes());
        let _vec = hasher.finalize().to_vec();

        let mut s = String::new();
        for b in _vec {
            s.push_str(&format!("{:02x}", b));
        }
        s
    };

    let current = chrono::Utc::now();
    let _result = userscollection
        .insert_one(
            doc! {
                "username": sanatize(&request.username),
                "password": &hashed_password,
                "quota": data.config.users.default_user_quota as i32,
                "privileges": vec![Privileges::CreateFile, Privileges::DeleteFile, Privileges::DeleteUser],
                "api_key": &api_key,
                "created_at": current,
            },
            None,
        )
        .await
        .unwrap();

    let _api_key = api_key.clone();

    web::block(move || std::fs::create_dir_all(format!("{}/{}", data.config.files.storage_path, _api_key)).unwrap()).await?;

    Ok(HttpResponse::Created().json(UserResponse {
        username: request.username.to_string(),
        password: hashed_password,
        api_key,
        privileges: vec![
            Privileges::CreateFile,
            Privileges::DeleteFile,
            Privileges::DeleteUser,
        ],
        created_at: current.to_rfc3339(),
    }))
}

/// DELETE: /api/v1/users
/// 
/// Delete a user
/// # Returns:
/// * `HttpResponse::Ok()` - User deleted
/// * `HttpResponse::NotFound()` - User not found
/// * `HttpResponse::Forbidden()` - User does not have sufficient privileges
/// # Parameters (JSON body):
/// * `api_key` - String
/// * `username` - String
pub async fn delete(
    data: web::Data<AppState>,
    request: web::Json<DeleteRequest>,
) -> Result<impl Responder> {
    log::debug("DELETE: /api/v1/users");
    let userscollection = data.database.collection::<Document>("users");

    let user = userscollection
        .find_one(doc! {"username": &request.username}, None)
        .await
        .unwrap();

    if user == None {
        return Ok(HttpResponse::NotFound().json(MessageResponse {
            message: "User not found".to_string(),
        }));
    }
    let user = user.unwrap();

    //? check if the api key has the privilage Privlages::GlobalDeleteUser
    if check_privilege(&user.clone(), Privileges::GlobalDeleteUser).await? {
        let _result = userscollection
            .delete_one(doc! {"username": &request.username}, None)
            .await
            .unwrap();

        let _result = web::block(move || {
            std::fs::remove_dir_all(format!("storage/{}", request.api_key)).unwrap()
        });

        return Ok(HttpResponse::Ok().json(MessageResponse {
            message: "User deleted".to_string(),
        }));
    }

    //? check if the username has the privilage Privlages::DeleteUser
    if check_privilege(&user, Privileges::DeleteUser).await? {
        let _result = userscollection
            .delete_one(doc! {"username": &request.username}, None)
            .await
            .unwrap();

        let _result = web::block(move || {
            std::fs::remove_dir_all(format!("storage/{}", request.api_key)).unwrap()
        });

        return Ok(HttpResponse::Ok().json(MessageResponse {
            message: "User deleted".to_string(),
        }));
    }

    Ok(HttpResponse::Forbidden().json(MessageResponse {
        message: "You don't have the privileges to delete this user".to_string(),
    }))
}
