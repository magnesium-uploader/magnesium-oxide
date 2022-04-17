use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct Privileges: u32 {
        // Normal user permissions
        const CREATE_FILE = 0x01;
        const DELETE_FILE = 0x02;
        const DELETE_USER = 0x04;
        // Elevated user permissions
        const UNLIMITED_QUOTA = 0x08;
        // Administrative user permissions
        const GLOBAL_DELETE_FILE = 0x10;
        const GLOBAL_DELETE_USER = 0x20;
    }
}

pub struct PrivilegesBuilder {
    privileges: Vec<Privileges>,
}

impl PrivilegesBuilder {
    pub fn new() -> PrivilegesBuilder {
        PrivilegesBuilder {
            privileges: Vec::new(),
        }
    }

    pub fn from_vec(privileges: Vec<Privileges>) -> PrivilegesBuilder {
        PrivilegesBuilder { privileges }
    }

    pub fn add(mut self, privilege: Privileges) -> PrivilegesBuilder {
        self.privileges.push(privilege);
        self
    }

    pub fn build(self) -> Privileges {
        let mut privileges = Privileges::empty();
        for privilege in self.privileges {
            privileges |= privilege;
        }
        privileges
    }
}

#[test]
fn test_privileges_builder() {
    // Ensure that the builder works as expected
    let builder: PrivilegesBuilder = PrivilegesBuilder::new()
        .add(Privileges::CREATE_FILE)
        .add(Privileges::DELETE_FILE)
        .add(Privileges::DELETE_USER);

    let privileges = builder.build();
    println!("{:?}", &privileges);
    assert_eq!(
        privileges,
        Privileges::CREATE_FILE | Privileges::DELETE_FILE | Privileges::DELETE_USER
    );

    // Ensure the bits match
    println!("{:#04x}", &privileges.bits);
    assert_eq!(privileges.bits(), 0x07);

    let builder: PrivilegesBuilder =
        PrivilegesBuilder::from_vec(vec![Privileges::CREATE_FILE, Privileges::DELETE_FILE]);

    let privileges = builder.build();
    println!("{:?}", &privileges);
    assert_eq!(
        privileges,
        Privileges::CREATE_FILE | Privileges::DELETE_FILE
    );

    // Ensure the bits match
    println!("{:#04x}", &privileges.bits);
    assert_eq!(privileges.bits(), 0x03);
}

/// Responce for the POST: /api/v1/users/{user_id} endpoint
/// # Fields:
///   * `username` - String
///   * `password` - String
///   * `privileges` - Privileges
///   * `api_key` - String
///   * `created_at` - String
#[derive(Serialize)]
pub struct UserResponse {
    pub username: String,
    pub password: String,
    pub privileges: u32,
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
    pub self_key: String,
    pub user_key: String,
}

/// Response (JSON body) for the POST: /api/v1/files endpoint
/// # Fields:
///   * `name` - String
///   * `size` - String
///   * `url` - String
///   * `deletion_key` - String
///   * `key` - String
///   * `nonce` - String
#[derive(Serialize, Deserialize, Debug, Default)]
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
