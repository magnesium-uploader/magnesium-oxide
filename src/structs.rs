use bson::{doc, oid::ObjectId, serde_helpers::chrono_datetime_as_bson_datetime};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::hashing::hash_string;

/// Simple struct for seralized use of authorization tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationHeader {
    /// An optional token to be used for authorization.
    pub authorization: Option<String>,
}

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

/// This module contains all user-related structs used in the API.
pub mod users {
    use super::*;

    /// # User
    /// This struct holds the information for a user in the database.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct User {
        /// User's unique ObjectId
        pub _id: ObjectId,
        /// Username
        pub username: String,
        /// Email
        pub email: String,
        /// Hashed password
        pub p_hash: String, //? ARGON2 hash
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
                p_hash: password.into(),
                email: email.into(),
                quota: User::default_quota(),
                token: hash_string(token.into()),
                privileges: Privileges::default(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }

        /// Generates a default UserQuota.
        pub fn default_quota() -> UserQuota {
            UserQuota {
                used: 0,
                // TODO: Make default available quota configurable (currently set to 1GB)
                available: 1024 * 1024 * 1024 * 8, // 8,589,934,592 bits
            }
        }

        /// Generates an API token for the user
        pub fn generate_token() -> String {
            Uuid::new_v4().to_string()
        }
    }

    /// # UserQuota
    /// This struct holds the information for a user's quota.
    /// * `used`: The amount of bytes used by the user
    /// * `available`: The amount of bytes available to the user (`used` - `available` = quota left)
    #[derive(Debug, Serialize, Deserialize)]
    pub struct UserQuota {
        /// Total size of all files uploaded by the user
        pub used: i64,
        /// Available space for the user
        pub available: i64,
    }

    /// # UserCreateRequest
    /// This struct holds the information for a user creation request.
    /// * `username`: Username
    /// * `password`: Password
    /// * `email`: Email
    #[derive(Debug, Serialize, Deserialize)]
    pub struct UserCreateRequest {
        /// The username of the user you are creating
        pub username: String,
        /// The hashed password of the user you are creating
        pub p_hash: String,
        /// The email of the user you are creating
        pub email: String,
    }

    /// # UserIdRequest
    /// This is a very simple struct for requesting a User's ObjectID for use in other requests.
    pub struct UserIdRequest {
        /// ObjectID of the user you are requesting in hexadecimal.
        pub id: String,
    }
}

/// This module contains all file-related structs used in the API.
pub mod files {
    use super::*;

    /// # File
    /// This struct is used to represent a file in the database and is used to
    /// serialize and deserialize the file data.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct File {
        /// The unique ObjectId of the file
        pub _id: ObjectId,
        /// The name of the file
        pub filename: String,
        /// The mime type of the file
        pub mimetype: String,
        /// The uploader's unique ObjectId
        pub uploader: ObjectId,
        /// The SHA3-512 hash of the file
        pub hash: String,
        /// The deletion token of the file (hashed)
        pub dkey: String,
        /// The size in bytes of the file
        pub size: i64,
        /// The DateTime<Utc> when the file was uploaded
        #[serde(with = "chrono_datetime_as_bson_datetime")]
        pub created_at: DateTime<Utc>,
    }

    /// Request arguments for getting a file
    /// # Arguments
    /// * `key`: The decryption key of the file in base64
    /// * `nonce`: The nonce of the file in base64
    #[derive(Debug, Deserialize)]
    pub struct FileGetRequest {
        /// Decryption key (base64)
        pub key: String,
        /// Nonce (base64)
        pub nonce: String,
    }

    /// Request arguments for deleting a file
    /// # Arguments
    /// * `hash` - The SHA3-512 hash of the file
    /// * `dkey` - The deletion key of the file
    #[derive(Debug, Serialize, Deserialize)]
    pub struct FileDeleteRequest {
        /// Hash of the file to delete
        pub hash: String,
        /// Deletion key
        pub dkey: String,
    }
}
