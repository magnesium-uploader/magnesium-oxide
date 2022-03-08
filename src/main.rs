use std::error::Error;
use std::fs::File;
use std::io::Read;

use actix_web::{App, HttpServer, web};
use mongodb::{Client, Database, options::ClientOptions};
use serde::Deserialize;

mod routes;
mod log;

#[derive(Deserialize, Clone)]
pub struct ConfigUser {
    default_user_quota: usize,
}

#[derive(Deserialize, Clone)]
pub struct ConfigFiles {
    storage_path: String,
    max_upload_size: usize,
}

#[derive(Deserialize, Clone)]
pub struct ConfigMongo {
    uri: String,
    database: String,
}


#[derive(Deserialize, Clone)]
pub struct Config {
    mongo: ConfigMongo,
    files: ConfigFiles,
    users: ConfigUser,
}

impl Config {
    fn new(file_path: &str) -> Result<Config, Box<dyn Error>> {
        let mut config_file = File::open(file_path)?;
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}

#[derive(Clone)]
pub struct AppState {
    database: Database,
    config: Config,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Parse a connection string into an options struct.
    let config = match Config::new("config.toml") {
        Ok(config) => {config},
        Err(_) => {
            log::critical("Could not parse config.toml");
            std::process::exit(1);
        },
    };
    let client_options = match ClientOptions::parse(&config.mongo.uri).await {
        Ok(client_options) => {client_options},
        Err(_) => {
            log::critical("Could not parse mongo uri");
            std::process::exit(1);
        },
    };
    let client = match Client::with_options(client_options) {
        Ok(client) => {client},
        Err(_) => {
            log::critical("Could not connect to mongo");
            std::process::exit(1);
        },
    };
    let _database = client.database(&config.mongo.database);
    let appstate = AppState {
        database: _database,
        config,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(appstate.clone()))
            .service(routes::v1::files::get_file)
            .service(routes::v1::files::delete_file)
            .service(
                // Create two scopes
                // outline: /api/v1/{users or files}                
                web::scope("/api").service(
                    web::scope("/v1").service(
                        web::scope("/users")
                            .route("", web::post().to(routes::v1::users::create))
                            .route("", web::delete().to(routes::v1::users::delete))
                    ).service(
                        web::scope("/files")
                            .route("", web::post().to(routes::v1::files::upload))
                    )
                )
            )
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
