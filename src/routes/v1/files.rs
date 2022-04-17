use std::io::{Read, Write};

use actix_multipart::Multipart;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use aes_gcm_siv::{
    aead::{Aead, NewAead},
    Aes256GcmSiv, Key, Nonce,
};
use futures_util::TryStreamExt as _;
use mongodb::bson::{doc, Document};
use rand::Rng;
use sha3::{Digest, Sha3_512};

use crate::log;
use crate::routes::v1::types::{FileDeleteQuery, FileGetQuery, FileResponse, MessageResponse};
use crate::routes::v1::utils::check_quota;
use crate::{
    crypto::{c_delete_one, c_find_one, c_replace_one},
    AppState,
};

use super::types::Privileges;
use super::utils::hash_to_string;

/// POST: /api/v1/files
///
/// Upload and encrypt a file with AES256-GCM-SIV
/// # Returns:
/// * `HttpResponse::Created()` - File uploaded
/// * `HttpResponse::Unauthorized()` - User does not have the correct privileges OR Credentials are missing/invalid
/// * `HttpResponse::BadRequest()` - File is too large OR User has met their quota OR No file was uploaded
/// # Parameters (Multipart form):
/// * `file` - File to upload
/// # Headers:
/// * `Authorization` - A string that contains the user's API key
/// * `zws` - Whether or not to use zws
pub async fn upload(
    data: web::Data<AppState>,
    mut payload: Multipart,
    content: HttpRequest,
) -> Result<HttpResponse, Error> {
    log::debug("POST: /api/v1/files");
    let files_collection = data.database.collection::<Document>("files");
    let users_collection = data.database.collection::<Document>("users");
    let mut response = FileResponse::default();

    while let Some(mut field) = payload.try_next().await.unwrap() {
        let auth = match content.headers().get("authorization") {
            Some(o) => o.to_str().unwrap(),
            None => {
                return Ok(HttpResponse::Unauthorized().json(MessageResponse {
                    message: "No Authorization Header".to_string(),
                }));
            }
        };

        let user = c_find_one(
            &users_collection,
            &doc! {"api_key": auth.to_string()},
            &data.config,
        )
        .await?;

        if user == None {
            return Ok(HttpResponse::Unauthorized().json(MessageResponse {
                message: "Invalid credentials".to_string(),
            }));
        }

        let mut user = user.unwrap();
        let privs = Privileges::from_bits_truncate(user.get_i32("privileges").unwrap() as u32);

        if !privs.contains(Privileges::CREATE_FILE) {
            return Ok(HttpResponse::Unauthorized().json(MessageResponse {
                message: "Your account lacks the privileges to upload files".to_string(),
            }));
        }

        let filename = &field
            .content_disposition()
            .get_filename()
            .unwrap()
            .to_string();

        let file_ext_regex = regex::Regex::new(r"(?i)^.*\.([a-z]{1,5})$").unwrap();

        let file_ext = file_ext_regex
            .captures(filename)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();

        let content_type = &field.content_type().to_string();

        let mut vec: Vec<i64> = Vec::with_capacity(32);
        for _ in 0..vec.capacity() {
            vec.push(rand::random());
        }

        let key = rand::thread_rng().gen::<[u8; 32]>();
        let nonce = rand::thread_rng().gen::<[u8; 12]>();
        let deletion_key =
            base64::encode_config(rand::thread_rng().gen::<[u8; 8]>(), base64::URL_SAFE_NO_PAD);

        let nonce = Nonce::from(nonce);
        let cipher = Aes256GcmSiv::new(Key::from_slice(&key));

        let mut bytes = Vec::new();
        while let Some(chunk) = field.try_next().await? {
            bytes.extend_from_slice(&chunk);
        }

        let size = bytes.len() as i64;
        if size > data.config.files.max_upload_size {
            return Ok(HttpResponse::BadRequest().json(MessageResponse {
                message: "File is too large".to_string(),
            }));
        }

        if !check_quota(data.as_ref(), &user, size).await? {
            return Ok(HttpResponse::BadRequest().json(MessageResponse {
                message: "You have reached your quota".to_string(),
            }));
        }

        let hash = hash_to_string(&bytes);

        user.insert("used", doc! {"used": size + user.get_i64("used").unwrap()});

        let file_doc = doc! {
            "hash": &hash,
            "name": filename.to_string(),
            "size": bytes.len() as i64,
            "type": content_type.to_string(),
            "deletion_key": &deletion_key,
            "uploaded": chrono::Utc::now(),
            "uploader": user.get_object_id("_id").unwrap(),
        };

        let ciphertext = cipher.encrypt(&nonce, bytes.as_ref()).unwrap();

        let mut file = std::fs::File::create(format!(
            "{}/{}/{}{}",
            data.config.files.storage_path, &auth, &hash, ".mgo"
        ))
        .unwrap();

        web::block(move || file.write_all(&ciphertext).map(|_| file)).await??; // write the bytes

        // Update the user's used space
        c_replace_one(
            &users_collection,
            &doc! {"_id": user.get_object_id("_id").unwrap()},
            &user,
            &data.config,
        )
        .await?;

        // Insert the file into the database
        c_replace_one(
            &files_collection,
            &doc! {"hash": &hash},
            &file_doc,
            &data.config,
        )
        .await?;

        let nonce = base64::encode_config(&nonce, base64::URL_SAFE_NO_PAD);
        let key = base64::encode_config(&key, base64::URL_SAFE_NO_PAD);

        log::info(format!(
            "{} uploaded {} ({} bytes) [{}]",
            auth, &filename, &size, &content_type
        ));

        if let Some(zws) = content.headers().get("zws") {
            if zws.to_str().unwrap() == "true" {
                response.name = format!(
                    "\u{200d}{}.{}",
                    super::utils::base64_to_zws(&hash),
                    file_ext
                );
                response.key = super::utils::base64_to_zws(&response.key);
                response.nonce = super::utils::base64_to_zws(&response.nonce);
            }
        }

        // Return the file
        response = FileResponse {
            name: format!("{}.{}", &hash, &file_ext),
            size: bytes.len().to_string(),
            url: data.config.network.return_address.to_string(),
            deletion_key,
            key,
            nonce,
        };
    }

    Ok(HttpResponse::Created().json(response))
}

/// GET: /{hash}/delete/
///
/// Deletes a file from the database and the file storage
/// # Returns:
/// * `HttpResponse::Ok()` - File deleted successfully
/// * `HttpResponse::NotFound()` - File not found
/// * `HttpResponse::Unauthorized()` - Invalid deletion key
/// # Parameters (Query String):
/// * `deletion_key` - Deletion key
#[get("/{name}/delete")]
pub async fn delete_file(
    data: web::Data<AppState>,
    path: web::Path<(String,)>,
    query: web::Query<FileDeleteQuery>,
) -> Result<HttpResponse, Error> {
    log::debug(&format!("GET: /api/v1/files/{}/delete", &path.0));
    let files_collection = data.database.collection::<Document>("files");
    let users_collection = data.database.collection::<Document>("users");

    let path = path.0.clone();

    let mut hash = path.split('.').next().unwrap().to_string();

    if hash.starts_with('\u{200d}') {
        hash = super::utils::zws_to_base64(&hash);
    }

    let file = c_find_one(&files_collection, &doc! {"hash": &hash}, &data.config).await?;
    if file == None {
        return Ok(HttpResponse::NotFound().json(MessageResponse {
            message: "File not found".to_string(),
        }));
    }
    let file = file.unwrap();
    println!("{:?}", file);

    let user = c_find_one(
        &users_collection,
        &doc! {"_id": &file.get("uploader").unwrap().as_object_id().unwrap()},
        &data.config,
    )
    .await?;
    let deletion_key = base64::decode_config(&query.deletion_key, base64::URL_SAFE_NO_PAD).unwrap();
    let file_key = base64::decode_config(
        file.get("deletion_key").unwrap().as_str().unwrap(),
        base64::URL_SAFE_NO_PAD,
    )
    .unwrap();

    if deletion_key != file_key {
        return Ok(HttpResponse::Unauthorized().json(MessageResponse {
            message: "Invalid deletion key".to_string(),
        }));
    }

    let user = user.unwrap();

    //println!("{}", format!("{}/{}/{}.mgo", data.config.files.storage_path, user.get("api_key").unwrap().to_string().replace("\"", ""), &hash));

    let _hash = hash.clone();
    let _conf = data.config.clone();

    web::block(move || {
        std::fs::remove_file(format!(
            "{}/{}/{}.mgo",
            data.config.files.storage_path,
            user.get("api_key").unwrap().to_string().replace('\"', ""),
            &hash
        ))
    })
    .await??;

    c_delete_one(&files_collection, &doc! {"hash": &_hash}, &_conf).await?;

    Ok(HttpResponse::Ok().body("Successfully deleted"))
}

/// GET: /{hash}
///
/// Decrypts a file and returns it
/// # Returns:
/// * `HttpResponse::Ok()` - File decrypted and uploaded successfully
/// * `HttpResponse::NotFound()` - File not found
/// # Parameters (Query String):
/// * `key` - File key
/// * `nonce` - File nonce
#[get("/{name}")]
pub async fn get_file(
    data: web::Data<AppState>,
    path: web::Path<(String,)>,
    query: web::Query<FileGetQuery>,
) -> Result<HttpResponse, Error> {
    let files_collection = data.database.collection::<Document>("files");
    let users_collection = data.database.collection::<Document>("users");

    let path = path.0.clone();

    let mut hash = path.split('.').next().unwrap().to_string();
    let mut key = query.key.clone();
    let mut nonce = query.nonce.clone();

    drop(query);

    if hash.starts_with('\u{200d}') {
        hash.remove(0);
        hash = super::utils::zws_to_base64(&hash);
        key = super::utils::zws_to_base64(&key);
        nonce = super::utils::zws_to_base64(&nonce);
    }

    log::debug(&format!("GET: /api/v1/files/{}", hash));
    let key = base64::decode_config(key, base64::URL_SAFE_NO_PAD).unwrap();
    let nonce = base64::decode_config(nonce, base64::URL_SAFE_NO_PAD).unwrap();

    let doc = c_find_one(&files_collection, &doc! {"hash": &hash}, &data.config).await?;
    if doc == None {
        return Ok(HttpResponse::NotFound().json(MessageResponse {
            message: "File not found".to_string(),
        }));
    }

    let doc = doc.unwrap();
    let uploader = doc.get("uploader").unwrap();
    let user = c_find_one(&users_collection, &doc! {"_id": uploader}, &data.config).await?;

    let user = user.unwrap();
    let api_key = user.get("api_key").unwrap().as_str().unwrap();

    let mut file =
        std::fs::File::open(format!("./storage/{}/{}{}", api_key, &hash, ".mgo")).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();

    let cipher = Aes256GcmSiv::new(Key::from_slice(&key));
    let nonce: [u8; 12] = nonce
        .try_into()
        .unwrap_or_else(|v: Vec<u8>| panic!("Invalid nonce: {:?}", v));
    let nonce = Nonce::from(nonce);
    let plaintext = cipher.decrypt(&nonce, bytes.as_ref()).expect("decrypt");

    Ok(HttpResponse::Ok()
        .insert_header((
            "Content-Disposition",
            format!("filename={}", doc.get("name").unwrap().as_str().unwrap()),
        ))
        .content_type(doc.get("type").unwrap().as_str().unwrap())
        .body(plaintext))
}
