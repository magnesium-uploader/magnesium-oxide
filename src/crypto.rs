use std::error::Error;
use std::io::{Read, Write};

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

pub async fn encrypt(content: String) -> Result<Vec<u8>, Box<dyn Error>> {
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

pub async fn decrypt(ciphertext: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let data = get_key()?;
    let _key = &data[0];
    let _nonce = &data[1];

    let key = (0..32).map(|i| _key[i]).collect::<Vec<u8>>();
    let nonce = (0..12).map(|i| _nonce[i]).collect::<Vec<u8>>();

    let nonce = Nonce::from_slice(&nonce);
    let cipher = Aes256GcmSiv::new(Key::from_slice(&key));

    let bytes = ciphertext.as_bytes();

    let plaintext = cipher.decrypt(nonce, bytes).unwrap();

    Ok(plaintext)
}