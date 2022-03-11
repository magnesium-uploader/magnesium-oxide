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
pub struct ConfigUser {
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
    pub mongo: ConfigMongo,
    pub files: ConfigFiles,
    pub users: ConfigUser,
    pub network: ConfigNetwork,
}

impl Config {
    pub fn new(file_path: &str) -> Result<Config, Box<dyn Error>> {
        let mut config_file = File::open(file_path)?;
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}