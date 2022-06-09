use std::str::FromStr;

use actix_web::{web::Form, Error, HttpRequest, HttpResponse, Result};

use bson::{doc, oid::ObjectId, serde_helpers::chrono_datetime_as_bson_datetime};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs::{self};
use uuid::Uuid;

use crate::{modules::hashing::hash_string, AppState};

bitflags::bitflags! {
    /// Bitflags! struct for user privileges
    #[derive(Serialize, Deserialize)]
    pub struct Privileges: u32 {
        /// User has full access to the API
        const ADMIN = 1;
        /// User can upload files, and delete their own files
        const USER = 2;
    }
}

impl Default for Privileges {
    fn default() -> Self {
        Privileges::USER
    }
}

/// User subquota struct
#[derive(Debug, Serialize, Deserialize)]
pub struct UserQuota {
    /// Total size of all files uploaded by the user
    pub used: i64,
    /// Available space for the user
    pub available: i64,
}

/// User struct
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    /// User's unique ObjectId
    pub _id: ObjectId,
    /// Username
    pub username: String,
    /// Email
    pub email: String,
    /// Hashed password
    pub password: String, //? SHA3-512 hash
    /// Quota for the user
    pub quota: UserQuota,
    /// Privileges bitflags
    pub privileges: Privileges,
    /// API Token
    pub token: String, //? SHA3-512 hash
    /// The DateTime<Utc> when the user was created
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    /// The DateTime<Utc> when the user last uploaded a file
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Creates a new user from `username`, `email`, `password`, and `token`.
    pub fn from<T>(username: T, password: T, email: T, token: T) -> User
    where
        T: Into<String>,
    {
        User {
            _id: ObjectId::new(),
            username: username.into(),
            password: hash_string(password.into()),
            email: email.into(),
            quota: User::default_quota(),
            token: hash_string(token.into()),
            privileges: Privileges::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Generates a default UserQuota with 100MB of available space
    pub fn default_quota() -> UserQuota {
        UserQuota {
            used: 0,
            available: 100 * 1024 * 1024,
        }
    }

    /// Generates an API token for the user
    pub fn generate_token() -> String {
        Uuid::new_v4().to_string()
    }
}

/// Endpoint options for creating a new user
#[derive(Debug, Serialize, Deserialize)]
pub struct UserRequest {
    /// The username of the user you are creating
    pub username: String,
    /// The password of the user you are creating
    pub password: String,
    /// The email of the user you are creating
    pub email: String,
}

/// Endpoint for creating a new user in the database
pub async fn create_user(
    request: HttpRequest,
    data: Form<UserRequest>,
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

/// Endpoint options for viewing a user
pub struct UserIdRequest {
    /// the ObjectID (in hex) of the user you are viewing
    pub id: String,
}

/// Endpoint for viewing a user in the database
pub async fn get_user(
    request: HttpRequest,
    data: Form<UserIdRequest>,
) -> Result<HttpResponse, Error> {
    let state = request.app_data::<AppState>().unwrap();
    let users = state.database.collection::<User>("users");

    let auth_header = match request.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap(),
        None => {
            return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
        }
    };

    let requester = users
        .find_one(doc! {"token": auth_header}, None)
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
