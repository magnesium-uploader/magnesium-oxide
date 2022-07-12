use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Options for using S3 compatible storage.
#[derive(Clone, Debug)]
pub struct S3Options {
    /// The S3 bucket name.
    pub bucket: String,
    /// The S3 region.
    pub region: String,
    /// The S3 endpoint.
    pub endpoint: String,
    /// The S3 access key.
    pub access_key: String,
    /// The S3 secret key.
    pub secret_key: String,
}

/// Enum storing the different types of storage modules.
/// Possible values are:
/// - `local`: Local storage module.
/// - `s3`: S3 storage module. (TODO)
#[derive(Clone, Debug)]
pub enum Storage {
    /// The local storage module, used for storing files locally at the path specified in the enum.
    Local(String),
    /// The S3 storage module, used for storing files in S3. (TODO)
    S3(S3Options),
}

impl Storage {
    /// Returns the path to the storage module.
    pub fn path(&self) -> Result<&str, &'static str> {
        match self {
            Storage::Local(path) => Ok(path),
            #[allow(unreachable_patterns)]
            _ => Err("This storage module is not a local storage module"),
        }
    }

    /// Gets the bytes of a file from the storage module.
    pub async fn get_file(
        &self,
        uid: &str,
        hash: &str,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        match self {
            Storage::Local(ref local) => {
                let path = format!("{}/{}/{}.mgo", local, uid, hash);
                let mut file = tokio::fs::File::open(path).await?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes).await?;
                Ok(Bytes::from(bytes))
            }
            Storage::S3(ref _s3) => {
                todo!("S3 storage module")
            }
        }
    }

    /// Saves a file to the storage module.
    pub async fn put_file(
        &self,
        uid: &str,
        hash: &str,
        bytes: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Storage::Local(ref local) => {
                let path = format!("{}/{}/{}.mgo", local, uid, hash);
                let mut file = tokio::fs::File::create(path).await?;
                file.write_all(bytes).await?;
                Ok(())
            }
            Storage::S3(ref _s3) => {
                todo!("S3 storage module")
            }
        }
    }

    /// Deletes a file from the storage module.
    pub async fn remove_file(
        &self,
        uid: &str,
        hash: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Storage::Local(ref local) => {
                let path = format!("{}/{}/{}.mgo", local, uid, hash);
                tokio::fs::remove_file(path).await?;
                Ok(())
            }
            Storage::S3(ref _s3) => {
                todo!("S3 storage module")
            }
        }
    }

    /// Checks if a file exists in the storage module.
    pub async fn exists(&self, uid: &str, hash: &str) -> bool {
        match self {
            Storage::Local(ref local) => {
                let path = format!("{}/{}/{}.mgo", local, uid, hash);
                tokio::fs::metadata(path).await.is_ok()
            }
            Storage::S3(ref _s3) => {
                todo!("S3 storage module")
            }
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        Storage::Local(String::from("data"))
    }
}
