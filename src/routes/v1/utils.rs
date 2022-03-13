use std::error::Error;

use bson::Document;
use futures_util::TryStreamExt;
use mongodb::bson::doc;

use crate::AppState;
use crate::routes::v1::types::Privileges;

/// Check the users quota
/// # Returns:
/// * `Result<bool, Box<dyn Error>>` - A result containing the a boolean indicating if the user has enough quota
/// # Parameters:
/// * `data` - A reference to the application state
/// * `user` - A bson document containing the user's information
/// * `upload_size` - The size of the file to be uploaded
pub async fn check_quota(data: &AppState, user: &Document, upload_size: usize) -> Result<bool, Box<dyn Error>> {
    if check_privilege(user, Privileges::UnlimitedQuota).await? {
        return Ok(true);
    }

    let files_collection = data.database.collection::<Document>("files");
    // Count the sizes of all the files owned by the user

    let mut _result = files_collection
        .find(doc! {"uploader": user.get_object_id("_id").unwrap()}, None)
        .await
        .unwrap();

    let mut total_size = 0;
    while let Some(file) = _result.try_next().await? {
        total_size += file.get_i32("size").unwrap() as usize;
    }

    // Check if the user has enough space to upload the file
    if total_size + upload_size > user.get_i32("quota").unwrap() as usize {
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
pub async fn check_privileges(user: &Document, privileges: &[Privileges]) -> Result<bool, Box<dyn Error>> {
    let user_privileges = user.get_array("privileges").unwrap()
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

/// Check a single privilege (use check_privileges for multiple privileges)
/// # Returns:
/// * `Result<bool, Box<dyn Error>>` - A result containing the a boolean indicating if the user has the privilege
/// # Parameters:
/// * `user` - A bson document containing the user's information
/// * `privilege` - A single privilege to check against the user's privileges
pub async fn check_privilege(user: &Document, privilege: Privileges) -> Result<bool, Box<dyn Error>> {
    let user_privileges = user.get_array("privileges").unwrap()
        .iter()
        .map(|privilege| privilege.as_str().unwrap())
        .collect::<Vec<&str>>();

    let mut _vec: Vec<Privileges> = Vec::new();

    for i in user_privileges {
        _vec.push(str_to_privilege(i));
    }

    if !_vec.contains(&privilege) {
        return Ok(false);
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
  '0',
  '1',
  '2',
  '3',
  '4',
  '5',
  '6',
  '7',
  '8',
  '9',
  'a',
  'b',
  'c',
  'd',
  'e',
  'f',
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
        let byte = u8::from_str_radix(&hex[i..i+2], 16).unwrap();
        string.push(byte as char);
    }

    let mut bytes = Vec::new();
    for i in string.chars() {
        bytes.push(i as u8);
    }

    let base64 = base64::encode_config(&bytes, base64::URL_SAFE_NO_PAD);
    base64
}

/// Typecast a string to a Privilege
/// # Returns:
/// * `Privileges` - A Privilege
/// # Parameters:
/// * `privilege` - A string repersenation of a Privilege
#[allow(dead_code)] // error: box too large
pub fn str_to_privilege(privilege: &str) -> Privileges {
    match privilege {
        "create_file" => Privileges::CreateFile,
        "delete_file" => Privileges::DeleteFile,
        "delete_user" => Privileges::DeleteUser,
        "global_delete_file" => Privileges::GlobalDeleteFile,
        "global_delete_user" => Privileges::GlobalDeleteUser,
        "unlimited_quota" => Privileges::UnlimitedQuota,
        _ => panic!("Invalid privilege"),
    }
}