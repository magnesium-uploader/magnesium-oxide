use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Enum storing the different types of storage modules.
/// Possible values are:
/// - `local`: Local storage module.
/// - `s3`: S3 storage module. (TODO)
#[derive(Clone, Debug)]
pub enum StorageModule {
    /// The local storage module, used for storing files locally at the path specified in the enum.
    Local(String),
    // S3(S3Options),
}

impl StorageModule {
    /// Returns the path to the storage module.
    pub fn path(&self) -> Result<&str, &'static str> {
        match self {
            StorageModule::Local(path) => Ok(path),
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
            StorageModule::Local(ref local) => {
                let path = format!("{}/{}/{}.mgo", local, uid, hash);
                let mut file = tokio::fs::File::open(path).await?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes).await?;
                Ok(Bytes::from(bytes))
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
            StorageModule::Local(ref local) => {
                let path = format!("{}/{}/{}.mgo", local, uid, hash);
                let mut file = tokio::fs::File::create(path).await?;
                file.write_all(bytes).await?;
                Ok(())
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
            StorageModule::Local(ref local) => {
                let path = format!("{}/{}/{}.mgo", local, uid, hash);
                tokio::fs::remove_file(path).await?;
                Ok(())
            }
        }
    }

    /// Checks if a file exists in the storage module.
    pub async fn exists(&self, uid: &str, hash: &str) -> bool {
        match self {
            StorageModule::Local(ref local) => {
                let path = format!("{}/{}/{}.mgo", local, uid, hash);
                tokio::fs::metadata(path).await.is_ok()
            }
        }
    }
}

impl Default for StorageModule {
    fn default() -> Self {
        StorageModule::Local(String::from("data"))
    }
}

/// Struct containing the storage module.
#[derive(Clone, Default, Debug)]
pub struct Storage {
    /// The storage module.
    pub module: StorageModule,
}
