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