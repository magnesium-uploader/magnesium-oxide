use actix_web::{web, HttpResponse, Responder, Result};
use mongodb::{
    bson::{doc, Document},
    Database,
};
use regex::Regex;
use sha3::{Digest, Sha3_512};

use crate::crypto::{c_find_one, c_insert_one};

use crate::log;
use crate::routes::v1::types::{
    MessageResponse, Privileges, UserRequest, UserResponse,
};
use crate::AppState;

use super::types::PrivilegesBuilder;

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

    if c_find_one(
        &userscollection,
        &doc! {"username": &request.username},
        &data.config,
    )
    .await?
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
    let user_privileges = PrivilegesBuilder::new()
        .add(Privileges::CREATE_FILE)
        .add(Privileges::DELETE_FILE)
        .add(Privileges::DELETE_USER)
        .build()
        .bits();

    c_insert_one(
        &userscollection,
        &doc! {
            "username": sanatize(&request.username),
            "password": &hashed_password,
            "quota": data.config.users.default_user_quota as i64,
            "used": 0_i64,
            "privileges": user_privileges,
            "api_key": &api_key,
            "created_at": current,
        },
        &data.config,
    )
    .await?;

    let _api_key = api_key.clone();

    web::block(move || {
        std::fs::create_dir_all(format!("{}/{}", data.config.files.storage_path, _api_key)).unwrap()
    })
    .await?;

    Ok(HttpResponse::Created().json(UserResponse {
        username: request.username.to_string(),
        password: hashed_password,
        api_key,
        privileges: user_privileges,
        created_at: current.to_rfc3339(),
    }))
}
