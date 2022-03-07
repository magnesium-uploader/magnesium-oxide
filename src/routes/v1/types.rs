use serde::{Deserialize, Serialize};


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

//impl trait bson convert for Privileges
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

//impl partialeq for privileges
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



#[derive(Serialize)]
pub struct UserResponse {
    pub username: String,
    pub password: String,
    pub privileges: Vec<Privileges>,
    pub api_key: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct UserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub api_key: String,
    pub username: String,
}

/*
{"file": {
    "name": "test.hgo",
    "hash": "test",
    "size": "123",
    "url": "https://localhost:7221/api/v1/storage/download/test?key=test&iv=test"
    },
    "key": "test",
    "nonce": "test"
}
*/

#[derive(Serialize, Deserialize)]
pub struct FileResponse {
    //? File Metadata
    pub hash: String,
    pub name: String,
    pub size: String,
    pub url: String,

    //? Encryption Metadata
    pub deletion_key: String,
    pub key: String,
    pub nonce: String,
}

#[derive(Deserialize)]
pub struct FileGetQuery {
    pub key: String,
    pub nonce: String
}

#[derive(Deserialize)]
pub struct FileDeleteQuery {
    pub deletion_key: String,
}