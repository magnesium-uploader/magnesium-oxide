use serde::{Deserialize, Serialize};
use std::io::Read;
use std::io::Write;
use toml;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub uri: String,
    pub db_name: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct LocalStorageConfig {
    pub enabled: bool,
    pub path: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct S3StorageConfig {
    pub enabled: bool,
    pub bucket: String,
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    pub local: LocalStorageConfig,
    pub s3: S3StorageConfig,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub database: DatabaseConfig,
}

impl Config {
    pub fn new(server: ServerConfig, storage: StorageConfig, database: DatabaseConfig) -> Config {
        Config {
            server,
            storage,
            database,
        }
    }

    fn from_toml(toml: &str) -> Result<Config, toml::de::Error> {
        let config = toml::from_str::<Config>(toml)?;
        Ok(config)
    }

    fn to_toml(&self) -> String {
        toml::to_string(&self).unwrap()
    }

    pub fn from_file(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let mut file = std::fs::File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;
        let config = Config::from_toml(&contents)?;
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = std::fs::File::create(path)?;
        let toml = self.to_toml();
        file.write_all(toml.as_bytes())?;
        Ok(())
    }

    pub fn get_or_create(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        if std::path::Path::new(path).exists() {
            Config::from_file(path)
        } else {
            let config = Config::default();
            config
                .to_file(path)
                .expect("Failed to create a default config file, check permissions");

            println!(
                "\x1b[31m[Magnesium] Please edit the default config file ({}) and restart the program.\x1b[0m",
                path
            );

            std::process::exit(0);
        }
    }
}
