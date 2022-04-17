use std::error::Error;

use bson::Document;

use mongodb::bson::doc;
use sha3::{Digest, Sha3_512};

use crate::crypto::c_find;
use crate::routes::v1::types::Privileges;
use crate::AppState;

/// Check the users quota
/// # Returns:
/// * `Result<bool, Box<dyn Error>>` - A result containing the a boolean indicating if the user has enough quota
/// # Parameters:
/// * `data` - A reference to the application state
/// * `user` - A bson document containing the user's information
/// * `upload_size` - The size of the file to be uploaded
pub async fn check_quota(
    _data: &AppState,
    user: &Document,
    upload_size: i64,
) -> Result<bool, Box<dyn Error>> {
    let privs = Privileges::from_bits_truncate(user.get_i32("privileges").unwrap() as u32);
    if privs.contains(Privileges::UNLIMITED_QUOTA) {
        return Ok(true);
    }

    let quota = user.get_i64("quota").unwrap();
    let used = user.get_i64("used").unwrap();
    let remaining = quota - used;
    if remaining < upload_size {
        return Ok(false);
    }

    Ok(true)
}

//its in the box now.

/// Check multiple privileges
/// # Returns:
/// * `Result<bool, Box<dyn Error>>` - A result containing the a boolean indicating if the user has the privilege
/// # Parameters:
/// * `user` - A bson document containing the user's information
/// * `privilege` - An array of privileges to check against the user's privileges
#[allow(dead_code)]
pub async fn check_privileges(
    user: &Document,
    privileges: &[Privileges],
) -> Result<bool, Box<dyn Error>> {
    let user_privileges = user
        .get_array("privileges")
        .unwrap()
        .iter()
        .map(|privilege| privilege.as_str().unwrap())
        .collect::<Vec<&str>>();

    let mut _vec: Vec<Privileges> = Vec::new();

    for i in user_privileges {
        _vec.push(str_to_privilege(i));
    }

    let privileges = privileges.to_vec();
    for i in privileges {
        if !_vec.contains(&i) {
            return Ok(false);
        }
    }

    Ok(true)
}

const ZWS: [char; 16] = [
    '\u{e006c}', // 0
    '\u{e006d}', // 1
    '\u{e006e}', // 2
    '\u{e006f}', // 3
    '\u{e0070}', // 4
    '\u{e0071}', // 5
    '\u{e0072}', // 6
    '\u{e0073}', // 7
    '\u{e0074}', // 8
    '\u{e0075}', // 9
    '\u{e0076}', // A
    '\u{e0077}', // B
    '\u{e0078}', // C
    '\u{e0079}', // D
    '\u{e007a}', // E
    '\u{e007f}', // F
];

const CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

pub fn base64_to_zws(input: &str) -> String {
    let decode = base64::decode_config(input, base64::URL_SAFE_NO_PAD).unwrap();

    let mut hex = String::new();
    for i in decode {
        hex.push_str(&format!("{:02x}", i));
    }

    let hex = hex.to_lowercase();
    let mut zws_string = String::new();
    for (_, c) in hex.chars().enumerate() {
        let index = CHARS.iter().position(|&x| x == c).unwrap();
        zws_string.push_str(&ZWS[index].to_string());
    }

    zws_string
}

pub fn zws_to_base64(input: &str) -> String {
    let mut hex = String::new();
    for i in input.chars() {
        let index = ZWS.iter().position(|&x| x == i).unwrap();
        hex.push_str(&CHARS[index].to_string());
    }

    let mut string = String::new();
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).unwrap();
        string.push(byte as char);
    }

    let mut bytes = Vec::new();
    for i in string.chars() {
        bytes.push(i as u8);
    }

    base64::encode_config(&bytes, base64::URL_SAFE_NO_PAD)
}

/// Typecast a string to a Privilege
/// # Returns:
/// * `Privileges` - A Privilege
/// # Parameters:
/// * `privilege` - A string repersenation of a Privilege
#[allow(dead_code)] // error: box too large
pub fn str_to_privilege(privilege: &str) -> Privileges {
    match privilege {
        "create_file" => Privileges::CREATE_FILE,
        "delete_file" => Privileges::DELETE_FILE,
        "delete_user" => Privileges::DELETE_USER,
        "global_delete_file" => Privileges::GLOBAL_DELETE_FILE,
        "global_delete_user" => Privileges::GLOBAL_DELETE_USER,
        "unlimited_quota" => Privileges::UNLIMITED_QUOTA,
        _ => panic!("Invalid privilege"),
    }
}

pub fn hash_to_string(bytes: &Vec<u8>) -> String {
    let mut hasher = Sha3_512::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    let mut hash_string = String::with_capacity(64);
    for byte in hash.as_slice() {
        hash_string.push_str(&format!("{:02x}", byte));
    }
    hash_string
}
