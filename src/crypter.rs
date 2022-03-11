use rand::Rng;
use std::io::{Write, Read};
use std::error::Error;
use aes_gcm_siv::{
    aead::{Aead, NewAead},
    Aes256GcmSiv, Key, Nonce,
};
use super::log;

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

            return get_key()
        }
    };

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let data = buf.split(|&x| x == 0).collect::<Vec<&[u8]>>();

    let key = base64::decode(data[0])?;
    let nonce = base64::decode(data[1])?;
    Ok(vec! [key, nonce])
}
// https://www.mongodb.com/basics/bson
pub async fn encrypt(content: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let data = get_key()?;
    let _key =  &data[0];
    let _nonce = &data[1];

    // bad code. don't do this.

    let mut key: [u8; 32] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    ];
    let mut nonce: [u8; 12] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
    ];

    for i in 0.._key.len() {
        key[i] = _key[i];
    };

    for i in 0.._nonce.len() {
        nonce[i] = _nonce[i];
    };

    log::debug(format!("{:?}", key).as_str());
    log::debug(format!("{:?}", nonce).as_str());

    let nonce = Nonce::from_slice(&nonce);
    let cipher = Aes256GcmSiv::new(Key::from_slice(&key));
    
    let bytes = content.as_bytes();

    let ciphertext = cipher.encrypt(nonce, bytes.as_ref()).unwrap();
    
    Ok(ciphertext)
}

pub async fn decrypt(ciphertext: String) -> Result<Vec<u8>, Box<dyn Error>>  {
    let data = get_key()?;
    let _key =  &data[0];
    let _nonce = &data[1];

    // bad code. don't do this.

    let mut key: [u8; 32] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    ];
    let mut nonce: [u8; 12] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
    ];

    for i in 0.._key.len() {
        key[i] = _key[i];
    };

    for i in 0.._nonce.len() {
        nonce[i] = _nonce[i];
    };

    log::debug(format!("{:?}", key).as_str());
    log::debug(format!("{:?}", nonce).as_str());

    let nonce = Nonce::from_slice(&nonce);
    let cipher = Aes256GcmSiv::new(Key::from_slice(&key));
    
    let bytes = ciphertext.as_bytes();

    let plaintext = cipher.decrypt(nonce, bytes).unwrap();

    Ok(plaintext)
}