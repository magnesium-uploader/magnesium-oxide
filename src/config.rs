use serde::Deserialize;
use std::io::Read;
use std::fs::File;
use std::error::Error;

#[derive(Deserialize, Clone)]
pub struct ConfigNetwork {
    pub bind_address: String,
    pub bind_port: u16,
    pub return_address: String,
}

#[derive(Deserialize, Clone)]
pub struct ConfigUsers {
    pub default_user_quota: usize,
}

#[derive(Deserialize, Clone)]
pub struct ConfigFiles {
    pub storage_path: String,
    pub max_upload_size: usize,
}

#[derive(Deserialize, Clone)]
pub struct ConfigMongo {
    pub uri: String,
    pub database: String,
}

#[derive(Deserialize, Clone)]
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
}