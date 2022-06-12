use actix_multipart::Multipart;
use actix_web::{
    web::{Path, Query},
    Error, HttpRequest, HttpResponse, Result,
};
use base64::URL_SAFE_NO_PAD;
use bson::{doc, oid::ObjectId};
use bytes::{Bytes, BytesMut};
use chrono::Utc;
use futures_util::{StreamExt, TryStreamExt};
use serde_json::json;
use tokio::fs::*;

use uuid::Uuid;

use crate::{
    modules::{
        crypto::{decrypt_bytes, encrypt_bytes, generate_key, EncryptionKey},
        hashing::{hash_bytes, hash_string},
    },
    structs::users::User,
    structs::{
        files::{File, FileDeleteRequest, FileGetRequest},
        Privileges,
    },
    AppState,
};

/// Endpoint for uploading files to magnesium-oxide using ShareX
pub async fn upload_file(request: HttpRequest, mut data: Multipart) -> Result<HttpResponse> {
    let state = request.app_data::<AppState>().unwrap();

    let files = state.database.collection::<File>("files");
    let users = state.database.collection::<User>("users");

    let auth_header = request.headers().get("Authorization");

    if auth_header.is_none() {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    }

    let token_hash = hash_string(auth_header.unwrap().to_str().unwrap());

    let uploader = users
        .find_one(
            doc! {
                "token": token_hash
            },
            None,
        )
        .await
        .unwrap();

    if uploader.is_none() {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
    }

    let uploader = uploader.unwrap();

    if !uploader.privileges.contains(Privileges::USER) {
        return Ok(
            HttpResponse::Unauthorized().body("Your privileges are not sufficient to upload files")
        );
    }

    let mut file_name = String::new();
    let mut file_mimetype = String::new();
    let mut file_bytes = vec![];

    // Retrieve the file from the multipart stream
    while let Some(mut field) = data.try_next().await.unwrap() {
        while let Some(chunk) = field.next().await {
            file_bytes.extend_from_slice(&chunk?);
        }

        if field.name() != "file" {
            return Ok(HttpResponse::BadRequest().body("Invalid file"));
        }

        file_name = field
            .content_disposition()
            .get_filename()
            .unwrap()
            .to_string();
        file_mimetype = field.content_type().to_string();
    }

    let file_hash = hash_bytes(&file_bytes);
    let file_size = file_bytes.len() as i64;

    let crypto = generate_key();
    let file_bytes = match encrypt_bytes(&crypto, &BytesMut::from(file_bytes.as_slice())) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().body("Failed to encrypt file"));
        }
    };

    // TODO: Enforce maximum upload size
    // if file_size > state.config.max_file_size {
    //     return Ok(HttpResponse::BadRequest().body("File is too large"));
    // }

    if uploader.quota.used + file_size >= uploader.quota.available {
        return Ok(HttpResponse::BadRequest()
            .body("The file you are trying to upload would exceed your available quota"));
    }

    let file_path = format!(
        "{}/{}/{}.mgo",
        state.config.storage.path,
        uploader._id.to_hex(),
        file_hash
    );

    match write(file_path, file_bytes).await {
        Ok(_) => {}
        Err(_) => {
            return Ok(HttpResponse::InternalServerError()
                .body("Failed to upload file, please try again later"));
        }
    }

    let dkey = Uuid::new_v4().to_string();

    let file = File {
        _id: ObjectId::new(),
        filename: file_name.clone(),
        mimetype: file_mimetype,
        uploader: uploader._id,
        hash: file_hash.clone(),
        dkey: hash_string(&dkey),
        size: file_size,
        created_at: Utc::now(),
    };

    let key_str = base64::encode_config(crypto.key, URL_SAFE_NO_PAD);
    let nonce_str = base64::encode_config(crypto.nonce, URL_SAFE_NO_PAD);

    let existing_file = files
        .find_one(doc! {"hash": &file_hash}, None)
        .await
        .unwrap();

    if existing_file.is_some() {
        files
            .delete_one(doc! {"_id": existing_file.unwrap()._id}, None)
            .await
            .unwrap();
        files.insert_one(file, None).await.unwrap();

        return Ok(HttpResponse::Ok().json(json!({
            "hash": file_hash,
            "ext": file_name.split('.').last().unwrap(),
            "key": key_str,
            "nonce": nonce_str,
            "dkey": &dkey,
        })));
    }

    users
    .update_one(
        doc! {"_id": uploader._id},
        doc! {"$set": {"quota.used": uploader.quota.used + file_size, "updated_at": file.created_at}},
        None,
    )
    .await
        .unwrap();

    files.insert_one(file, None).await.unwrap();

    Ok(HttpResponse::Created().json(json!({
        "hash": file_hash,
        "ext": file_name.split('.').last().unwrap(),
        "key": key_str,
        "nonce": nonce_str,
        "dkey": dkey
    })))
}

/// Endpoint for deleting files from magnesium-oxide using ShareX
pub async fn delete_file(
    request: HttpRequest,
    data: Query<FileDeleteRequest>,
) -> Result<HttpResponse> {
    let state = request.app_data::<AppState>().unwrap();
    let dkey = hash_string(&data.dkey);

    let files = state.database.collection::<File>("files");

    let file = files
        .find_one(doc! {"hash": &data.hash}, None)
        .await
        .unwrap();

    if file.is_none() {
        return Ok(HttpResponse::NotFound()
            .body("The specified file does not exist or your deletion key is invalid"));
    }

    let file = file.unwrap();

    if dkey != file.dkey {
        return Ok(HttpResponse::Unauthorized().body("Invalid deletion key"));
    }

    match remove_file(format!(
        "{}/{}/{}.mgo",
        state.config.storage.path,
        file.uploader.to_hex(),
        file.hash
    ))
    .await
    {
        Ok(_) => {
            files
                .delete_one(doc! {"_id": file._id}, None)
                .await
                .unwrap();
            Ok(HttpResponse::Ok().body("File deleted"))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("Failed to delete file")),
    }
}

/// Endpoint for viewing files from magnesium-oxide using ShareX
pub async fn get_file(
    request: HttpRequest,
    auth: Query<FileGetRequest>,
    hash: Path<String>,
) -> Result<HttpResponse, Error> {
    let state = request.app_data::<AppState>().unwrap();

    let files = state.database.collection::<File>("files");

    let file = files
        .find_one(doc! {"hash": hash.split('.').next().unwrap()}, None)
        .await
        .unwrap();

    if file.is_none() {
        return Ok(HttpResponse::NotFound().body("File not found"));
    }

    let file = file.unwrap();

    let file_bytes = match read(format!(
        "{}/{}/{}.mgo",
        state.config.storage.path,
        file.uploader.to_hex(),
        file.hash
    ))
    .await
    {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().body("Failed to read file"));
        }
    };

    let file_bytes = Bytes::from(file_bytes);

    let key = base64::decode_config(&auth.key, URL_SAFE_NO_PAD).unwrap();
    let nonce = base64::decode_config(&auth.nonce, URL_SAFE_NO_PAD).unwrap();

    let crypto = EncryptionKey { key, nonce };

    let file_bytes = match decrypt_bytes(&crypto, &file_bytes) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().body("Failed to decrypt file"));
        }
    };

    //? Originally: Content-Disposition: attachment; filename="filename.ext"
    Ok(HttpResponse::Ok()
        .content_type(file.mimetype.clone())
        .append_header((
            "Content-Disposition",
            format!("filename=\"{}\"", file.filename),
        ))
        .body(file_bytes))
}
