use serde::{Deserialize, Serialize};

use std::io::Read;
use std::io::Write;

use toml;

/// Databse configuration
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    /// MongoDB URI
    pub uri: String,
    /// Database name
    pub db_name: String,
}

/// Application configuration
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    /// Server listening address
    pub host: String,
    /// Server listening port
    pub port: u16,
}

/// Storage configuration
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    /// The path where uploaded content will be stored
    pub path: String,
    /// The maximum size of an uploaded file
    pub max_size: u64,
}

/// General configuration
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Database configuration
    pub database: DatabaseConfig,
}

impl Config {
    /// Creates a new Config struct
    /// # Arguments
    /// * `server` - ServerConfig
    /// * `storage` - StorageConfig
    /// * `database` - DatabaseConfig
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

    /// Loads the configuration from a file
    pub fn from_file(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let mut file = std::fs::File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;
        let config = Config::from_toml(&contents)?;
        Ok(config)
    }

    /// Saves the configuration to a file
    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = std::fs::File::create(path)?;
        let toml = self.to_toml();
        file.write_all(toml.as_bytes())?;
        Ok(())
    }

    /// Creates a new Config at the given path, or loads it if it already exists
    /// # Arguments
    /// * `path` - Path to the config file
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
