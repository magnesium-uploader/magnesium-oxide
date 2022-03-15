use actix_web::{HttpResponse, Responder, Result, web};
use mongodb::{
    bson::{doc, Document},
    Database,
};
use regex::Regex;
use sha3::{Digest, Sha3_512};

use crate::crypto::{c_insert_one, c_find_one, c_delete_one, c_delete_many};

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

    if c_find_one(&userscollection, &doc! {"username": &request.username}, &data.config).await? != None {
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

    c_insert_one(
        &userscollection,
        &doc! {
            "username": sanatize(&request.username),
            "password": &hashed_password,
            "quota": data.config.users.default_user_quota as i32,
            "privileges": vec![Privileges::CreateFile, Privileges::DeleteFile, Privileges::DeleteUser],
            "api_key": &api_key,
            "created_at": current,
        },
        &data.config,
    ).await?;

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
    let filescollection = data.database.collection::<Document>("files");

    let user = c_find_one(&userscollection, &doc! {"username": &request.username}, &data.config).await?;

    if user == None {
        return Ok(HttpResponse::NotFound().json(MessageResponse {
            message: "User not found".to_string(),
        }));
    }
    let user = user.unwrap();

    let user_id = user.get_object_id("_id").unwrap();

    //? check if the api key has the privilage Privlages::GlobalDeleteUser
    if check_privilege(&user.clone(), Privileges::GlobalDeleteUser).await? {
        c_delete_one(&userscollection, &doc! {"_id": &user_id}, &data.config).await?;
        c_delete_many(&filescollection, &doc! {"uploader": &user_id}, &data.config).await?;

        let _result = web::block(move || {
            std::fs::remove_dir_all(format!("storage/{}", request.api_key)).unwrap()
        });

        return Ok(HttpResponse::Ok().json(MessageResponse {
            message: "User deleted".to_string(),
        }));
    }

    //? check if the username has the privilage Privlages::DeleteUser
    if check_privilege(&user, Privileges::DeleteUser).await? {
        c_delete_one(&userscollection, &doc! {"_id": &user_id}, &data.config).await?;
        c_delete_many(&filescollection, &doc! {"uploader": &user_id}, &data.config).await?;

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
