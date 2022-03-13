use std::io::{Read, Write};

use actix_multipart::Multipart;
use actix_web::{Error, get, HttpRequest, HttpResponse, web};
use aes_gcm_siv::{
    aead::{Aead, NewAead},
    Aes256GcmSiv, Key, Nonce,
};
use futures_util::TryStreamExt as _;
use mongodb::bson::{doc, Document};
use rand::Rng;
use sha3::{Digest, Sha3_512};

use crate::AppState;
use crate::log;
use crate::routes::v1::types::{FileDeleteQuery, FileGetQuery, FileResponse, MessageResponse};
use crate::routes::v1::utils::check_quota;

use super::{types::Privileges, utils::check_privilege};

/// POST: /api/v1/files
/// 
/// Upload and encrypt a file with AES256-GCM-SIV
/// # Returns:
/// * `HttpResponse::Created()` - File uploaded
/// * `HttpResponse::Unauthorized()` - User does not have the correct privileges OR Creditentials are missing/invalid
/// * `HttpResponse::BadRequest()` - File is too large OR User has met their quota OR No file was uploaded
/// # Parameters (Multipart form):
/// * `file` - File to upload
/// # Headers:
/// * `Authorization` - A string that contains the user's API key
/// * `zws` - Whether or not to use zws
pub async fn upload(data: web::Data<AppState>, mut payload: Multipart, content: HttpRequest) -> Result<HttpResponse, Error> {
    log::debug("POST: /api/v1/files");
    let mut res = FileResponse {
        name: "".to_string(),
        size: "".to_string(),
        deletion_key: "".to_string(),
        key: "".to_string(),
        nonce: "".to_string(),
        url: "".to_string(),
    };

    let files_collection = data.database.collection::<Document>("files");
    let users_collection = data.database.collection::<Document>("users");

    while let Some(mut field) = payload.try_next().await.unwrap() {
        let auth = content.headers().get("authorization").unwrap().to_str().unwrap();
        let user = users_collection.find_one(doc! {"api_key": auth.to_string()}, None).await.unwrap();
        if user == None {
            return Ok(HttpResponse::Unauthorized().json(MessageResponse {
                message: "Invalid credentials".to_string(),
            }));
        }
        let user = user.unwrap();

        if !check_privilege(&user, Privileges::CreateFile).await? {
            return Ok(HttpResponse::Unauthorized().json(MessageResponse {
                message: "Your account does not have the required privileges to upload files".to_string(),
            }));
        }

        let filename = &field.content_disposition().get_filename().unwrap().to_string();
        let file_ext_regex = regex::Regex::new(r"(?i)^.*\.([a-z]{1,5})$").unwrap();
        let file_ext = file_ext_regex.captures(filename).unwrap().get(1).unwrap().as_str();
        let content_type = &field.content_type().to_string();

        let mut vec: Vec<i64> = Vec::with_capacity(32);
        for _ in 0..vec.capacity() {
            vec.push(rand::random());
        }

        let key = rand::thread_rng().gen::<[u8; 32]>();
        let nonce = rand::thread_rng().gen::<[u8; 12]>();
        let deletion_key = base64::encode_config(rand::thread_rng().gen::<[u8; 8]>(), base64::URL_SAFE_NO_PAD);
        let nonce = Nonce::from(nonce);
        let cipher = Aes256GcmSiv::new(Key::from_slice(&key));

        let mut bytes = Vec::new(); // create a new vector to store the file
        while let Some(chunk) = field.try_next().await? { bytes.extend_from_slice(&chunk); } // read the file
        let size = bytes.len();

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

        let hash = {
            let mut hasher = Sha3_512::new();
            hasher.update(&bytes);
            let _vec = hasher.finalize().to_vec();

            let mut s = String::new();
            for b in _vec {
                s.push_str(&format!("{:02x}", b));
            }
            s
        };

        let doc = doc! {
            "hash": &hash,
            "name": filename.to_string(),
            "size": bytes.len() as u32,
            "type": content_type.to_string(),
            "deletion_key": &deletion_key,
            "uploaded": chrono::Utc::now(),
            "uploader": user.get("_id").unwrap().as_object_id(),
        };

        let mut file = std::fs::File::create(format!("{}/{}/{}{}", data.config.files.storage_path, &auth, &hash, ".hgo")).unwrap(); //create the file

        let ciphertext = cipher.encrypt(&nonce, bytes.as_ref()).unwrap(); // encrypt the bytes

        web::block(move || file.write_all(&ciphertext).map(|_| file)).await??; // write the bytes

        let options = mongodb::options::ReplaceOptions::builder()
            .upsert(true)
            .build();

        files_collection.replace_one(doc! {"hash": &hash}, doc, options).await.unwrap();

        log::info(format!("{} uploaded {} ({} bytes) [{}]", auth, &filename, &size, &content_type).as_str());
        let nonce = base64::encode_config(&nonce, base64::URL_SAFE_NO_PAD);
        let key = base64::encode_config(&key, base64::URL_SAFE_NO_PAD);

        res = FileResponse {
            name: format!("{}.{}", &hash, &file_ext),
            size: bytes.len().to_string(),
            url: data.config.network.return_address.to_string(),
            deletion_key,
            key,
            nonce,
        };

        if let Some(zws) = content.headers().get("zws") { // check discord owo
            if zws.to_str().unwrap() == "true" {
                res.name = format!("\u{200d}{}.{}", super::utils::base64_to_zws(&hash), file_ext);
                res.key = super::utils::base64_to_zws(&res.key);
                res.nonce = super::utils::base64_to_zws(&res.nonce);
            }
        }
    }

    if res.size == "" {
        return Ok(HttpResponse::BadRequest().json(MessageResponse {
            message: "No file uploaded".to_string(),
        }));
    }
    Ok(HttpResponse::Created().json(res))
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
pub async fn delete_file(data: web::Data<AppState>, path: web::Path<(String, )>, query: web::Query<FileDeleteQuery>) -> Result<HttpResponse, Error> {
    log::debug(&format!("GET: /api/v1/files/{}/delete", &path.0).to_string());
    let files_collection = data.database.collection::<Document>("files");
    let users_collection = data.database.collection::<Document>("users");

    let path = path.0.clone();

    let mut hash = path.split(".").next().unwrap().to_string();

    if hash.chars().nth(0).unwrap() == '\u{200d}' {
        hash.remove(0);
        hash = super::utils::zws_to_base64(&hash);
    }

    let file = files_collection.find_one(doc! {"hash": &hash}, None).await.unwrap();
    if file == None {
        return Ok(HttpResponse::NotFound().json(MessageResponse {
            message: "File not found".to_string(),
        }));
    }
    let file = file.unwrap();

    let user = users_collection.find_one(doc! {"_id": &file.get("uploader").unwrap().as_object_id().unwrap()}, None).await.unwrap();
    let deletion_key = base64::decode_config(&query.deletion_key, base64::URL_SAFE_NO_PAD).unwrap();

    let file_key = base64::decode_config(file.get("deletion_key").unwrap().as_str().unwrap(), base64::URL_SAFE_NO_PAD).unwrap();

    if deletion_key != file_key {
        return Ok(HttpResponse::Unauthorized().json(MessageResponse {
            message: "Invalid deletion key".to_string(),
        }));
    }

    let user = user.unwrap();

    //println!("{}", format!("{}/{}/{}.hgo", data.config.files.storage_path, user.get("api_key").unwrap().to_string().replace("\"", ""), &hash));

    let _hash = hash.clone();

    web::block(move || std::fs::remove_file(format!("{}/{}/{}.hgo", data.config.files.storage_path, user.get("api_key").unwrap().to_string().replace("\"", ""), &hash))).await??;

    files_collection.delete_one(doc! {"hash": &_hash.to_string()}, None).await.unwrap();

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
pub async fn get_file(data: web::Data<AppState>, path: web::Path<(String, )>, query: web::Query<FileGetQuery>) -> Result<HttpResponse, Error> {
    let files_collection = data.database.collection::<Document>("files");
    let users_collection = data.database.collection::<Document>("users");

    let path = path.0.clone();

    let mut hash = path.split(".").next().unwrap().to_string();
    let mut key = query.key.clone();
    let mut nonce = query.nonce.clone();

    drop(query);

    if hash.chars().nth(0).unwrap() == '\u{200d}' {
        hash.remove(0);
        hash = super::utils::zws_to_base64(&hash);
        key = super::utils::zws_to_base64(&key);
        nonce = super::utils::zws_to_base64(&nonce);
    }

    log::debug(&format!("GET: /api/v1/files/{}", hash).to_string());
    let key = base64::decode_config(key, base64::URL_SAFE_NO_PAD).unwrap();
    let nonce = base64::decode_config(nonce, base64::URL_SAFE_NO_PAD).unwrap();

    let doc = files_collection.find_one(doc! {"hash": &hash}, None).await.unwrap();
    if doc == None {
        return Ok(HttpResponse::NotFound().json(MessageResponse {
            message: "File not found".to_string(),
        }));
    }

    let doc = doc.unwrap();
    let uploader = doc.get("uploader").unwrap();
    let user = users_collection.find_one(doc! {"_id": uploader}, None).await.unwrap();

    let user = user.unwrap();
    let api_key = user.get("api_key").unwrap().as_str().unwrap();

    let mut file = std::fs::File::open(format!("./storage/{}/{}{}", api_key, &hash, ".hgo")).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();

    let cipher = Aes256GcmSiv::new(Key::from_slice(&key));
    let nonce: [u8; 12] = nonce.try_into().unwrap_or_else(|v: Vec<u8>| {
        panic!("Invalid nonce: {:?}", v)
    });
    let nonce = Nonce::from(nonce);
    let plaintext = cipher.decrypt(&nonce, bytes.as_ref()).expect("decrypt");

    Ok(HttpResponse::Ok()
    .insert_header(("Content-Disposition", format!("filename={}", doc.get("name").unwrap().as_str().unwrap())))
    .content_type(doc.get("type").unwrap().as_str().unwrap())
    .body(plaintext))
}
