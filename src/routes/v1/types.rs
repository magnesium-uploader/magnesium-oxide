use serde::{Deserialize, Serialize};

/// User Privlages enum
/// # Derives:
/// * `Clone`
/// * `Serialize`
/// # Fields:
/// * `CreateFile` - Create and upload a file
/// * `DeleteFile` - Delete a file made by the user that created it
/// * `DeleteUser` - Let the user delete themselves
///
/// * `GlobalDeleteFile` - Delete any file
/// * `GlobalDeleteUser` - Delete any user
/// * `UnlimitedQuota` - Unlimited upload quota
/// # Implementations:
/// * `From<Privileges> -> bson::Bson` - Convert a Privlage to bson::Bson
/// * `PartialEq -> bool` - Check if two Privlages are equal
#[derive(Serialize, Clone)]
pub enum Privileges {
    CreateFile,
    DeleteFile,
    DeleteUser,
    // Admin privileges (Global)
    GlobalDeleteFile,
    GlobalDeleteUser,
    UnlimitedQuota,
}

impl From<Privileges> for bson::Bson {
    fn from(privilege: Privileges) -> bson::Bson {
        match privilege {
            Privileges::CreateFile => bson::Bson::String("create_file".to_string()),
            Privileges::DeleteFile => bson::Bson::String("delete_file".to_string()),
            Privileges::DeleteUser => bson::Bson::String("delete_user".to_string()),
            Privileges::GlobalDeleteFile => bson::Bson::String("global_delete_file".to_string()),
            Privileges::GlobalDeleteUser => bson::Bson::String("global_delete_user".to_string()),
            Privileges::UnlimitedQuota => bson::Bson::String("unlimited_quota".to_string()),
        }
    }
}

impl PartialEq for Privileges {
    fn eq(&self, other: &Privileges) -> bool {
        match (self, other) {
            (Privileges::CreateFile, Privileges::CreateFile) => true,
            (Privileges::DeleteFile, Privileges::DeleteFile) => true,
            (Privileges::DeleteUser, Privileges::DeleteUser) => true,
            (Privileges::GlobalDeleteFile, Privileges::GlobalDeleteFile) => true,
            (Privileges::GlobalDeleteUser, Privileges::GlobalDeleteUser) => true,
            (Privileges::UnlimitedQuota, Privileges::UnlimitedQuota) => true,
            _ => false,
        }
    }
}

/// Responce for the POST: /api/v1/users/{user_id} endpoint
/// # Fields:
///   * `username` - String
///   * `password` - String
///   * `privileges` - Vec<Privileges>
///   * `api_key` - String
///   * `created_at` - String
#[derive(Serialize)]
pub struct UserResponse {
    pub username: String,
    pub password: String,
    pub privileges: Vec<Privileges>,
    pub api_key: String,
    pub created_at: String,
}

/// Request (JSON body) for the POST: /api/v1/users/{user_id} endpoint
/// # Fields:
///   * `username` - String
///   * `password` - String
#[derive(Deserialize)]
pub struct UserRequest {
    pub username: String,
    pub password: String,
}

/// General message response
/// # Fields:
///   * `message` - String
#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

/// Request (JSON body) for the DELETE: /api/v1/users/{user_id} endpoint
/// # Fields:
///   * `api_key` - String
///   * `username` - String
#[derive(Deserialize)]
pub struct DeleteRequest {
    pub api_key: String,
    pub username: String,
}

/// Response (JSON body) for the POST: /api/v1/files endpoint
/// # Fields:
///   * `name` - String
///   * `size` - String
///   * `url` - String
///   * `deletion_key` - String
///   * `key` - String
///   * `nonce` - String
#[derive(Serialize, Deserialize, Debug)]
pub struct FileResponse {
    //? File Metadata
    pub name: String,
    pub size: String,
    pub url: String,

    //? Encryption Metadata
    pub deletion_key: String,
    pub key: String,
    pub nonce: String,
}

/// Request (Query String) for the GET: /{hash} endpoint
/// # Fields:
///   * `key` - String
///   * `nonce` - String
#[derive(Deserialize)]
pub struct FileGetQuery {
    pub key: String,
    pub nonce: String,
}

/// Request (Query String) for the DELETE: /{hash} endpoint
/// # Fields:
///   * `deletion_key` - String
#[derive(Deserialize)]
pub struct FileDeleteQuery {
    pub deletion_key: String,
}