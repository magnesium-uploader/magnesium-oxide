use bson::{doc, oid::ObjectId, serde_helpers::chrono_datetime_as_bson_datetime};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::hashing::hash_string;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationHeader {
    pub authorization: Option<String>,
}

bitflags::bitflags! {

    #[derive(Serialize, Deserialize)]
    pub struct Privileges: u32 {
        const ADMIN = 1;
        const USER = 2;
    }
}

impl Default for Privileges {
    fn default() -> Self {
        Privileges::USER
    }
}

pub mod users {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct User {
        pub _id: ObjectId,
        pub username: String,
        pub email: String,
        pub password: String, //? SHA3-512 hash
        pub quota: UserQuota,
        pub privileges: Privileges,
        pub token: String, //? SHA3-512 hash
        #[serde(with = "chrono_datetime_as_bson_datetime")]
        pub created_at: DateTime<Utc>,
        #[serde(with = "chrono_datetime_as_bson_datetime")]
        pub updated_at: DateTime<Utc>,
    }

    impl User {
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

        pub fn default_quota() -> UserQuota {
            UserQuota {
                used: 0,

                available: 1024 * 1024 * 1024 * 8,
            }
        }

        pub fn generate_token() -> String {
            Uuid::new_v4().to_string()
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct UserQuota {
        pub used: i64,
        pub available: i64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct UserCreateRequest {
        pub username: String,
        pub password: String,
        pub email: String,
    }

    pub struct UserIdRequest {
        pub id: String,
    }
}

pub mod files {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct File {
        pub _id: ObjectId,
        pub filename: String,
        pub mimetype: String,
        pub uploader: ObjectId,
        pub hash: String,
        pub dkey: String,
        pub size: i64,
        #[serde(with = "chrono_datetime_as_bson_datetime")]
        pub created_at: DateTime<Utc>,
    }

    #[derive(Debug, Deserialize)]
    pub struct FileGetRequest {
        pub key: String,
        pub nonce: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct FileDeleteRequest {
        pub hash: String,
        pub dkey: String,
    }
}
