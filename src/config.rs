use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct ConfigNetwork {
    pub bind_address: String,
    pub bind_port: u16,
    pub return_address: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ConfigUsers {
    pub default_user_quota: usize,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ConfigFiles {
    pub storage_path: String,
    pub max_upload_size: usize,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ConfigMongo {
    pub uri: String,
    pub database: String,
    pub encryption: bool
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub network: ConfigNetwork,
    pub users: ConfigUsers,
    pub files: ConfigFiles,
    pub mongo: ConfigMongo,
}

impl Config {
    pub fn new(file_path: &str) -> Result<Config, Box<dyn Error>> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn make_default(file_path: &str) -> Result<(), Box<dyn Error>> {
        let default = Config {
            network: ConfigNetwork {
                bind_address: "127.0.0.1".to_string(),
                bind_port: 8080,
                return_address: "http://localhost:8080".to_string()
            },
            users: ConfigUsers {
                default_user_quota: 1024 * 1024 * 1024,
            },
            files: ConfigFiles {
                storage_path: "/storage".to_string(),
                max_upload_size: 1024 * 1024 * 1024,
            },
            mongo: ConfigMongo {
                uri: "".to_string(),
                database: "".to_string(),
                encryption: false
            },
        };
        let toml = toml::to_string(&default).unwrap();

        let mut buf = File::create(file_path)?;
        buf.write_all(toml.as_bytes())?;
        Ok(())
    }
}