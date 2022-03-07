use std::io::{Write, Read};
use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse, HttpRequest, get};
use aes_gcm_siv::{
    aead::{NewAead, Aead},
    Aes256GcmSiv, Key, Nonce,
};
use futures_util::TryStreamExt as _;
use rand::Rng;
use sha3::{Digest, Sha3_512};
use mongodb::bson::{Document, doc};

use crate::log;
use crate::routes::types::{FileResponse, MessageResponse, FileGetQuery, FileDeleteQuery};
use crate::routes::utils::check_quota;
use crate::AppState;

//define the autorization header

pub async fn upload(data: web::Data<AppState>, mut payload: Multipart, content: HttpRequest) -> Result<HttpResponse, Error> {    
    log::debug("POST: /api/v1/files/upload");
    let mut res = FileResponse {
        hash: "".to_string(),
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
        log::debug("Upload Started");
        let auth = content.headers().get("authorization").unwrap().to_str().unwrap();        
        let user = users_collection.find_one(doc!{"api_key": auth.to_string()}, None).await.unwrap();
        if user == None {
            return Ok(HttpResponse::Unauthorized().json(MessageResponse {
                message: "Invalid credentials".to_string(),
            }));
        }
        let user = user.unwrap();
        // A multipart/form-data stream has to contain `content_disposition`
        let filename = &field.content_disposition().get_filename().unwrap().to_string();
        let content_type = &field.content_type().to_string();

        // File::create is blocking operation, use threadpool
        let mut vec: Vec<i64> = Vec::with_capacity(32);
        for _ in 0..vec.capacity() {
            vec.push(rand::random());
        }

        let key = rand::thread_rng().gen::<[u8; 32]>();
        let nonce = rand::thread_rng().gen::<[u8; 12]>();
        let deletion_key = base64::encode_config(rand::thread_rng().gen::<[u8; 8]>(), base64::URL_SAFE_NO_PAD);
        let nonce = Nonce::from(nonce);
        let cipher = Aes256GcmSiv::new(Key::from_slice(&key));

        // create a vector of bytes
        let mut bytes = Vec::new();
        
        // Field in turn is stream of *Bytes* object without borrowing
        while let Some(chunk) = field.try_next().await? { bytes.extend_from_slice(&chunk); }

        log::debug("Upload Complete");

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

        let doc = doc!{
            "hash": &hash,
            "name": filename.to_string(),
            "size": bytes.len() as u32,
            "type": content_type.to_string(),
            "deletion_key": &deletion_key,
            "uploaded": chrono::Utc::now(),
            "uploader": user.get("_id").unwrap().as_object_id(),
        };

        let mut file = std::fs::File::create(format!("{}/{}/{}{}", data.config.files.storage_path, &auth ,&hash, ".hgo")).unwrap(); //create the file

        log::debug(bytes.len().to_string().as_str());
        let ciphertext = cipher.encrypt(&nonce, bytes.as_ref()).expect("encrypt"); // encrypt the bytes

        web::block(move || file.write_all(&ciphertext).map(|_| file)).await??; // write the bytes
        
        let options = mongodb::options::ReplaceOptions::builder()
            .upsert(true)
            .build();

        files_collection.replace_one(doc! {"hash": &hash}, doc, options).await.unwrap();
        log::info(format!("{} uploaded {} ({} bytes) [{}]", auth, &filename, &size, &content_type).as_str());
            
        let nonce = base64::encode_config(&nonce, base64::URL_SAFE_NO_PAD);
        let key = base64::encode_config(&key, base64::URL_SAFE_NO_PAD);
        res = FileResponse {
            name: format!("{}.hgo", &hash),
            hash,
            size: bytes.len().to_string(),
            url: "http://localhost:8080".to_string(),
            deletion_key,
            key,
            nonce,
        }
    }
    if res.size == "" {
        return Ok(HttpResponse::BadRequest().json(MessageResponse {
            message: "No file uploaded".to_string(),
        }));
    }
    Ok(HttpResponse::Created().json(res))
}

// /api/v1/files/{hash}/delete/?deletekey={deletekey}
#[get("/{hash}/delete")]
pub async fn delete_file(data: web::Data<AppState>, path: web::Path<(String, )>, query: web::Query<FileDeleteQuery>) -> Result<HttpResponse, Error> {
    log::debug(&format!("GET: /api/v1/files/{}/delete", path.0).to_string());
    let files_collection = data.database.collection::<Document>("files");
    let users_collection = data.database.collection::<Document>("users");

    let file = files_collection.find_one(doc!{"hash": path.0.to_string()}, None).await.unwrap();
    if file == None {
        return Ok(HttpResponse::NotFound().json(MessageResponse {
            message: "File not found".to_string(),
        }));
    }
    let file = file.unwrap();
    
    let user = users_collection.find_one(doc!{"_id": &file.get("uploader").unwrap().as_object_id().unwrap()}, None).await.unwrap();
    let deletion_key = base64::decode_config(&query.deletion_key, base64::URL_SAFE_NO_PAD).unwrap();

    let file_key = base64::decode_config(file.get("deletion_key").unwrap().as_str().unwrap(), base64::URL_SAFE_NO_PAD).unwrap();

    println!("{:?}", file.get("deletion_key").unwrap().as_str().unwrap());
    println!("{:?}", &query.deletion_key);

    if deletion_key != file_key{
        return Ok(HttpResponse::Unauthorized().json(MessageResponse {
            message: "Invalid deletion key".to_string(),
        }));
    }

    let user = user.unwrap();

    println!("{}", format!("{}/{}/{}.hgo", data.config.files.storage_path, user.get("api_key").unwrap().to_string().replace("\"", ""), &path.0));

    let _path = path.clone();

    web::block(move || std::fs::remove_file(format!("{}/{}/{}.hgo", data.config.files.storage_path, user.get("api_key").unwrap().to_string().replace("\"", ""), &path.0))).await??;

    files_collection.delete_one(doc! {"hash": &_path.0.to_string()}, None).await.unwrap();

    Ok(HttpResponse::Ok().body("Successfully deleted"))
}

// /api/v1/files/{hash}?key={key}&nonce={nonce}
#[get("/{hash}")]
pub async fn get_file(data: web::Data<AppState>, path: web::Path<(String, )>, query: web::Query<FileGetQuery>) -> Result<HttpResponse, Error> {
    let files_collection = data.database.collection::<Document>("files");
    let users_collection = data.database.collection::<Document>("users");
    
    let hash = path.into_inner().0;
    log::debug(format!("GET: /{}", &hash.as_str()).as_str());
    let key = base64::decode_config(&query.key, base64::URL_SAFE_NO_PAD).unwrap();
    let nonce = base64::decode_config(&query.nonce, base64::URL_SAFE_NO_PAD).unwrap();

    let doc = files_collection.find_one(doc!{"hash": hash.to_string()}, None).await.unwrap();
    if doc == None {
        return Ok(HttpResponse::NotFound().json(MessageResponse {
            message: "File not found".to_string(),
        }));
    }

    let doc = doc.unwrap();
    let uploader = doc.get("uploader").unwrap();
    let user = users_collection.find_one(doc!{"_id": uploader}, None).await.unwrap();

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

    Ok(HttpResponse::Ok().content_type(doc.get("type").unwrap().as_str().unwrap()).body(plaintext))
}