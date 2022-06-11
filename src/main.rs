//! # Magnesium Oxide
//! A Rust ShareX uploader made from the ground up with security in mind, providing millitary-grade encryption and a high-level API for uploading files to the server.

#![forbid(unsafe_code)]
#![warn(missing_docs)] // TODO: add docs

/// All modules used by the program
pub mod modules;
/// All routes used by the program
pub mod routes;

use actix_web::{
    web::{self, ServiceConfig},
    App, HttpServer,
};

use log::{debug, error, info};
use modules::config::Config;
use mongodb::{options::ClientOptions, Client, Database};
use routes::{api::v1::files::*, api::v1::users::*, index::*};
use tokio::fs;

/// The actix_web AppState struct
#[derive(Clone)]
pub struct AppState {
    /// The configuration refrenced in all routes
    pub config: Config,
    /// The MongoDB Database refrenced in all routes
    pub database: Database,
}

fn routes(cfg: &mut ServiceConfig) {
    cfg.route("/", web::get().to(index))
        .route("/api/v1/files", web::post().to(upload_file))
        .route("/api/v1/files/delete", web::get().to(delete_file))
        .route("/{hash}", web::get().to(get_file))
        .route("/api/v1/users", web::post().to(create_user));
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let _config = Config::get_or_create("config.toml").unwrap();
    let config = _config.clone(); //TODO: remove the need for cloning the config struct

    let client_options = match ClientOptions::parse(&config.database.uri).await {
        Ok(opt) => {
            debug!("Connecting to database...");
            opt
        }
        Err(_) => {
            error!("Failed to parse MongoDB URI");
            std::process::exit(1);
        }
    };

    let client = match Client::with_options(client_options) {
        Ok(client) => {
            info!("Connection established to MongoDB");
            client
        }
        Err(_) => {
            error!("Failed to connect to MongoDB");
            std::process::exit(1);
        }
    };

    let database = client.database(&config.database.db_name);

    fs::create_dir_all(&config.storage.path).await.unwrap();

    info!("Starting server...");
    HttpServer::new(move || {
        App::new()
            .app_data(AppState {
                config: config.clone(),
                database: database.clone(),
            })
            .configure(routes)
    })
    .bind(format!("{}:{}", _config.server.host, _config.server.port))
    .unwrap()
    .run()
    .await
    .unwrap();
}
