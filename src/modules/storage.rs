use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::config::S3StorageConfig;

#[derive(Clone, Debug)]
pub enum Storage {
    Local(String),
    S3(S3StorageConfig),
}

impl Storage {
    pub fn path(&self) -> Result<&str, &'static str> {
        match self {
            Storage::Local(path) => Ok(path),
            #[allow(unreachable_patterns)]
            _ => Err("This storage module is not a local storage module"),
        }
    }

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
