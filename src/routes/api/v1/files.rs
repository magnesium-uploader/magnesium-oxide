use actix_multipart::Multipart;
use actix_web::{
    web::{Path, Query},
    Error, HttpRequest, HttpResponse, Result,
};
use base64::URL_SAFE_NO_PAD;
use bson::{doc, oid::ObjectId};
use bytes::BytesMut;
use chrono::Utc;
use futures_util::{StreamExt, TryStreamExt};
use serde_json::json;

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

    let storage = state.storage.clone();
    let mut file_name = String::new();
    let mut file_mimetype = String::new();
    let mut file_bits = vec![];

    while let Some(mut field) = data.try_next().await.unwrap() {
        while let Some(chunk) = field.next().await {
            file_bits.extend_from_slice(&chunk?);
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

    let file_hash = hash_bytes(&file_bits);
    let file_size = file_bits.len() as i64;

    let crypto = generate_key();
    let file_bits = match encrypt_bytes(&crypto, &BytesMut::from(file_bits.as_slice())) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().body("Failed to encrypt file"));
        }
    };

    if uploader.quota.used + file_size >= uploader.quota.available {
        return Ok(HttpResponse::BadRequest()
            .body("The file you are trying to upload would exceed your available quota"));
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

    storage
        .put_file(file.uploader.to_hex().as_str(), &file_hash, &file_bits)
        .await?;

    let key_str = base64::encode_config(crypto.key, URL_SAFE_NO_PAD);
    let nonce_str = base64::encode_config(crypto.nonce, URL_SAFE_NO_PAD);

    let file_check = files
        .find_one(doc! {"hash": &file_hash}, None)
        .await
        .unwrap();

    if file_check.is_none() {
        files.insert_one(&file, None).await.unwrap();

        users
        .update_one(
            doc! {"_id": uploader._id},
            doc! {"$set": {"quota.used": uploader.quota.used + file_size, "updated_at": file.created_at}},
            None,
        ).await.unwrap();
    } else {
        files
            .delete_one(doc! {"_id": file_check.unwrap()._id}, None)
            .await
            .unwrap();
        files.insert_one(file, None).await.unwrap();
    }

    return Ok(HttpResponse::Created().json(json!({
        "hash": file_hash,
        "ext": file_name.split('.').last().unwrap(),
        "key": key_str,
        "nonce": nonce_str,
        "dkey": dkey
    })));
}

pub async fn delete_file(
    request: HttpRequest,
    data: Query<FileDeleteRequest>,
) -> Result<HttpResponse> {
    let state = request.app_data::<AppState>().unwrap();
    let files = state.database.collection::<File>("files");
    let storage = state.storage.clone();

    let dkey = hash_string(&data.dkey);

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

    storage
        .remove_file(file.uploader.to_hex().as_str(), &file.hash)
        .await?;

    files
        .delete_one(doc! {"_id": file._id}, None)
        .await
        .unwrap();

    Ok(HttpResponse::NoContent().body(""))
}

pub async fn get_file(
    request: HttpRequest,
    auth: Query<FileGetRequest>,
    hash: Path<String>,
) -> Result<HttpResponse, Error> {
    let state = request.app_data::<AppState>().unwrap();
    let files = state.database.collection::<File>("files");
    let storage = state.storage.clone();

    let hash = hash.into_inner();
    let hash = hash.split('.').next().unwrap();

    let file = match files.find_one(doc! {"hash": &hash}, None).await {
        Ok(file) => {
            if file.is_none() {
                return Ok(HttpResponse::NotFound().body("The specified file does not exist"));
            }
            file.unwrap()
        }
        Err(_) => {
            return Ok(
                HttpResponse::InternalServerError().body("Failed to retrieve file from database")
            );
        }
    };

    let file_bits = match storage
        .get_file(file.uploader.to_hex().as_str(), &file.hash)
        .await
    {
        Ok(bytes) => {
            let key = base64::decode_config(&auth.key, URL_SAFE_NO_PAD).unwrap();
            let nonce = base64::decode_config(&auth.nonce, URL_SAFE_NO_PAD).unwrap();

            let crypto = EncryptionKey { key, nonce };

            match decrypt_bytes(&crypto, &bytes) {
                Ok(dbytes) => dbytes,
                Err(_) => {
                    return Ok(HttpResponse::InternalServerError().body("Failed to decrypt file"));
                }
            }
        }
        Err(_) => {
            return Ok(HttpResponse::NotFound().body("The specified file does not exist"));
        }
    };

    Ok(HttpResponse::Ok()
        .content_type(file.mimetype.clone())
        .append_header((
            "Content-Disposition",
            format!("filename=\"{}\"", file.filename),
        ))
        .body(file_bits))
}
