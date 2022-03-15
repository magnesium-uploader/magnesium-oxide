use std::error::Error;
use std::io::{Read, Write};


use futures_util::{StreamExt};
use mongodb::{Collection, bson};

use aes_gcm_siv::{
    aead::{Aead, NewAead},
    Aes256GcmSiv, Key, Nonce,
};
use rand::Rng;




pub fn get_key() -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    // look for a file called "oxide.key" containing bytes
    // if it exists, read the bytes and use them as the key
    let mut file = match std::path::Path::exists(std::path::Path::new("./oxide.key")) {
        true => std::fs::File::open("oxide.key")?,
        false => {
            // if the file doesn't exist, generate a new key
            // and write it to the file
            let key = rand::thread_rng().gen::<[u8; 32]>();
            let nonce = rand::thread_rng().gen::<[u8; 12]>();
            let key = base64::encode(&key);
            let nonce = base64::encode(&nonce);

            let mut file = std::fs::File::create("oxide.key")?;

            file.write_all(key.as_bytes())?;
            file.write_all("\x00".as_bytes())?;
            file.write_all(nonce.as_bytes())?;

            println!("\x1b[1;31m
⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠
- We have detected that you are running mecury-oxide 
- for the first time, and have generated a key.
- Please back up oxide.key, or you risk total data loss.
⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠ ⚠
            \x1b[0m");

            return get_key();
        }
    };

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let data = buf.split(|&x| x == 0).collect::<Vec<&[u8]>>();

    let key = base64::decode(data[0])?;
    let nonce = base64::decode(data[1])?;
    Ok(vec![key, nonce])
}

pub async fn c_find_one(collection: &Collection::<bson::Document>, query: &bson::Document, config: &crate::config::Config) -> Result<Option<bson::Document>, Box<dyn Error>> {
    if !config.mongo.encryption {
        match collection.find_one(query.clone(), None).await? {
            Some(o) => {return Ok(Some(o))},
            None => {return Ok(None)},
        }
    } 

    let enc_query = bson_encrypt(&query.clone()).await.unwrap();

    //println!("{:?}", enc_query);
    //println!("{:?}", query);


    match collection.find_one(enc_query, None).await.unwrap() {
        Some(o) => return Ok(Some(bson_decrypt(&o).await.unwrap())),
        None => {
            return Ok(None);
        }
    };
}

pub async fn c_find(collection: &Collection::<bson::Document>, query: &bson::Document, config: &crate::config::Config) -> Result<Option<Vec<bson::Document>>, Box<dyn Error>> {
    if !config.mongo.encryption {
        match collection.find(query.clone(), None).await {
            Ok(mut o) => {
                let mut output = Vec::new();
                while let Some(object) = o.next().await {
                    let object = object?;
                    output.push(object);
                } 
                return Ok(Some(output));
            },
            Err(_) => {
                return Ok(None);
            }
        };
    }

    let enc_query = bson_encrypt(&query.clone()).await.unwrap();

    match collection.find(enc_query, None).await {
        Ok(mut o) => {
            let mut output = Vec::new();
            while let Some(object) = o.next().await {
                let object = object?;
                output.push(bson_decrypt(&object).await.unwrap());
            } 
            return Ok(Some(output));
        },
        Err(_) => { return Ok(None)},
    };
}

pub async fn c_insert_one(collection: &Collection::<bson::Document>, doc: &bson::Document, config: &crate::config::Config) -> Result<(), Box<dyn Error>> {
    if !config.mongo.encryption {
        collection.insert_one(doc.clone(), None).await?;
        return Ok(())
    } 
    
    let enc_doc = bson_encrypt(&doc.clone()).await?;
    collection.insert_one(enc_doc, None).await?;

    Ok(())
}

pub async fn c_replace_one(collection: &Collection::<bson::Document>, query: &bson::Document, doc: &bson::Document, config: &crate::config::Config) -> Result<(), Box<dyn Error>> {
    let options = mongodb::options::ReplaceOptions::builder()
        .upsert(true)
        .build();

    if !config.mongo.encryption {
        collection.replace_one(query.clone(), doc.clone(), options).await?;
        return Ok(())
    }

    let enc_doc = bson_encrypt(&doc.clone()).await?;
    let enc_query = bson_encrypt(&query.clone()).await?;
    collection.replace_one(enc_query, enc_doc, options).await?;
    
    Ok(())
}

pub async fn c_delete_one(collection: &Collection::<bson::Document>, query: &bson::Document, config: &crate::config::Config) -> Result<(), Box<dyn Error>> {
    if !config.mongo.encryption {
        collection.delete_one(query.clone(), None).await?;
        return Ok(())
    }

    let enc_query = bson_encrypt(&query.clone()).await?;
    collection.delete_one(enc_query, None).await?;

    Ok(())
}

pub async fn c_delete_many(collection: &Collection::<bson::Document>, query: &bson::Document, config: &crate::config::Config) -> Result<(), Box<dyn Error>> {
    if !config.mongo.encryption {
        collection.delete_many(query.clone(), None).await?;
        return Ok(())
    }

    let enc_query = bson_encrypt(&query.clone()).await?;
    collection.delete_many(enc_query, None).await?;

    Ok(())
}

async fn bson_encrypt(document: &bson::Document) -> Result<bson::Document, Box<dyn Error>> {
    let mut enc_document = bson::Document::new();

    for i in 0..document.keys().count() {
        let key = document.keys().nth(i).unwrap();
        let value = document.get(key).unwrap();

        let enc_value = encrypt_bson_element(value).await?;

        enc_document.insert(key, enc_value);
    }

    Ok(enc_document)
}

async fn bson_decrypt(enc_document: &bson::Document) -> Result<bson::Document, Box<dyn Error>> {
    let mut document = bson::Document::new();
    
    for i in 0..enc_document.keys().count() {
        let key = enc_document.keys().nth(i).unwrap();
        let value = enc_document.get(key).unwrap();

        let value = decrypt_bson_element(value).await?;

        document.insert(key, value);
    }

    Ok(document)
}

#[async_recursion::async_recursion(?Send)]
async fn encrypt_bson_element(element: &bson::Bson) -> Result<bson::Bson, Box<dyn Error>> {
    use bson::Bson;
    match element {
        Bson::ObjectId(o) => Ok(Bson::ObjectId(*o)),
        Bson::Int32(i) => Ok(Bson::String(
            base64::encode_config(encrypt(i.to_string()).await.unwrap(), base64::URL_SAFE_NO_PAD)
        )),
        Bson::Int64(i) => Ok(Bson::String(
            base64::encode_config(encrypt(i.to_string()).await.unwrap(), base64::URL_SAFE_NO_PAD)
        )),
        Bson::String(s) => Ok(Bson::String(
            base64::encode_config(encrypt(s.to_string()).await.unwrap(), base64::URL_SAFE_NO_PAD)
        )),
        Bson::Boolean(b) => Ok(Bson::Boolean(*b)),
        Bson::DateTime(d) => Ok(Bson::DateTime(*d)),
        Bson::Array(a) => {
            let mut enc_array = Vec::new();
            for element in a.iter() {
                enc_array.push(encrypt_bson_element(element).await?);
            }
            Ok(Bson::Array(enc_array))
        },
        _ => {
            Ok(Bson::String(element.to_string()))
        }
    }
}

#[async_recursion::async_recursion(?Send)]
async fn decrypt_bson_element(element: &bson::Bson) -> Result<bson::Bson, Box<dyn Error>> {
    use bson::Bson;
    match element {
        Bson::ObjectId(o) => Ok(Bson::ObjectId(*o)),
        Bson::String(s) => {
            let res = decrypt(base64::decode_config(s, base64::URL_SAFE_NO_PAD).unwrap()).await.unwrap();

            if let Ok(i) = res.parse::<i32>() {
                Ok(Bson::Int32(i))
            } else if let Ok(i) = res.parse::<i64>() {
                Ok(Bson::Int64(i))
            } else {
                Ok(Bson::String(res))
            }
        },
        Bson::Boolean(b) => Ok(Bson::Boolean(*b)),
        Bson::DateTime(d) => Ok(Bson::DateTime(*d)),
        Bson::Array(a) => {
            let mut dec_array = Vec::new();
            for element in a.iter() {
                dec_array.push(decrypt_bson_element(element).await?);
            }
            Ok(Bson::Array(dec_array))
        },
        _ => {
            Ok(Bson::String(element.to_string()))
        }
    }
}

async fn encrypt(content: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let data = get_key()?;
    let _key = &data[0];
    let _nonce = &data[1];

    let key = (0..32).map(|i| _key[i]).collect::<Vec<u8>>();
    let nonce = (0..12).map(|i| _nonce[i]).collect::<Vec<u8>>();

    let nonce = Nonce::from_slice(&nonce);
    let cipher = Aes256GcmSiv::new(Key::from_slice(&key));

    let bytes = content.as_bytes();

    let ciphertext = cipher.encrypt(nonce, bytes.as_ref()).unwrap();

    Ok(ciphertext)
}

async fn decrypt(ciphertext: Vec<u8>) -> Result<String, Box<dyn Error>> {
    let data = get_key()?;
    let _key = &data[0];
    let _nonce = &data[1];

    let key = (0..32).map(|i| _key[i]).collect::<Vec<u8>>();
    let nonce = (0..12).map(|i| _nonce[i]).collect::<Vec<u8>>();

    let nonce = Nonce::from_slice(&nonce);
    let cipher = Aes256GcmSiv::new(Key::from_slice(&key));
    
    let ciphertext = unsafe {
        std::slice::from_raw_parts(ciphertext.as_ptr(), ciphertext.len())
    };

    let plaintext = cipher.decrypt(nonce, ciphertext).unwrap();

    Ok(std::string::String::from_utf8(plaintext)?)
}