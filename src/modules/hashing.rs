use sha3::{Digest, Sha3_512};

pub fn hash_string<T: Into<String>>(input: T) -> String {
    let mut hasher = Sha3_512::default();
    hasher.update(input.into().as_bytes());
    let out = format!("{:x}", hasher.finalize());

    out
}

pub fn hash_bytes(input: &[u8]) -> String {
    let mut hasher = Sha3_512::default();
    hasher.update(input);

    let out = format!("{:x}", hasher.finalize());
    out
}
