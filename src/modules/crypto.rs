use std::fmt::Display;

use aes_gcm_siv::{
    aead::{Aead, NewAead},
    Aes256GcmSiv, Key, Nonce,
};
use bytes::{Bytes, BytesMut};
use rand::{rngs::OsRng, Rng};
use std::io::{Error, ErrorKind};

pub struct EncryptionKey {
    pub key: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl Display for EncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EncryptionKey {{ key: {:?}, nonce: {:?} }}",
            self.key, self.nonce
        )
    }
}

pub fn generate_key() -> EncryptionKey {
    let mut rng = OsRng::default();
    let key: [u8; 32] = rng.gen();
    let nonce: [u8; 12] = rng.gen();

    let nonce = Nonce::from_slice(&nonce).to_vec();
    let key = Key::from(key).to_vec();

    EncryptionKey { key, nonce }
}

pub fn encrypt_bytes(
    crypto: &EncryptionKey,
    data: &BytesMut,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let nonce = Nonce::from_slice(&crypto.nonce);
    let cipher = Aes256GcmSiv::new(Key::from_slice(&crypto.key));

    let data_crypt = match cipher.encrypt(nonce, data.as_ref()) {
        Ok(data) => BytesMut::from(data.as_slice()),
        Err(_) => {
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "Failed to encrypt data",
            )))
        }
    };

    Ok(data_crypt.freeze())
}

pub fn decrypt_bytes(
    crypto: &EncryptionKey,
    data: &Bytes,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let nonce = Nonce::from_slice(&crypto.nonce);
    let cipher = Aes256GcmSiv::new(Key::from_slice(&crypto.key));

    let data_decrypt = match cipher.decrypt(nonce, data.as_ref()) {
        Ok(data) => BytesMut::from(data.as_slice()),
        Err(_) => {
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "Failed to decrypt data",
            )))
        }
    };

    Ok(data_decrypt.freeze())
}

#[test]
fn test_crypto() {
    let crypto = generate_key();
    let data = BytesMut::from("Hello World!");

    println!("{}", crypto);

    let encrypted = match encrypt_bytes(&crypto, &data) {
        Ok(bytes) => {
            println!("{:?}", bytes);
            bytes
        }
        Err(err) => {
            println!("{:?}", err);
            panic!("Failed to encrypt data");
        }
    };

    let decrypted = match decrypt_bytes(&crypto, &encrypted) {
        Ok(bytes) => {
            println!("{:?}", bytes);
            bytes
        }
        Err(err) => {
            println!("{:?}", err);
            panic!("Failed to decrypt data");
        }
    };

    assert_eq!(data.as_ref(), decrypted.as_ref());
}
