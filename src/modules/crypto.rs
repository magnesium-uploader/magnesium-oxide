use std::fmt::Display;

use aes_gcm_siv::{
    aead::{Aead, NewAead},
    Aes256GcmSiv, Key, Nonce,
};
use bytes::{Bytes, BytesMut};
use rand::{rngs::OsRng, Rng};

/// Encryption key struct for use in the crypto functions.
pub struct EncryptionKey {
    /// The encryption key.
    pub key: Vec<u8>,
    /// The encryption nonce.
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

/// Generates a new EncryptionKey.
pub fn generate_key() -> EncryptionKey {
    let mut rng = OsRng::default();
    let key: [u8; 32] = rng.gen();
    let nonce: [u8; 12] = rng.gen();

    let nonce = Nonce::from_slice(&nonce).to_vec();
    let key = Key::from(key).to_vec();

    EncryptionKey { key, nonce }
}

/// Encrypts a Bytes object using the given EncryptionKey.
pub fn encrypt_bytes(
    crypto: &EncryptionKey,
    data: &BytesMut,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let nonce = Nonce::from_slice(&crypto.nonce);
    let cipher = Aes256GcmSiv::new(Key::from_slice(&crypto.key));

    let data_crypt = match cipher.encrypt(nonce, data.as_ref()) {
        Ok(data) => BytesMut::from(data.as_slice()),
        Err(_) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to encrypt data",
            )))
        }
    };

    Ok(data_crypt.freeze())
}

/// Decrypts a Bytes object using the given EncryptionKey.
pub fn decrypt_bytes(
    crypto: &EncryptionKey,
    data: &Bytes,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let nonce = Nonce::from_slice(&crypto.nonce);
    let cipher = Aes256GcmSiv::new(Key::from_slice(&crypto.key));

    let data_decrypt = match cipher.decrypt(nonce, data.as_ref()) {
        Ok(data) => BytesMut::from(data.as_slice()),
        Err(_) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to decrypt data",
            )))
        }
    };

    Ok(data_decrypt.freeze())
}

#[test]
fn test_crypto() {
    let crypto = generate_key();

    let data = BytesMut::from("Hello World".as_bytes());

    let encrypted = encrypt_bytes(&crypto, &data).unwrap();
    let decrypted = decrypt_bytes(&crypto, &encrypted).unwrap();

    assert_eq!(data, decrypted);
}