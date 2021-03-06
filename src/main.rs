//! # Magnesium Oxide
//! A Rust ShareX uploader made from the ground up with security in mind, providing millitary-grade encryption and a high-level API for uploading files to the server.

#![forbid(unsafe_code)]
#![warn(unreachable_pub, unused_qualifications)]

pub mod modules;
pub mod routes;
pub mod structs;

use actix_web::{
    web::{self, ServiceConfig},
    App, HttpServer,
};

use log::{debug, error, info};
use modules::{config::Config, storage::Storage};
use mongodb::{options::ClientOptions, Client, Database};
use routes::{api::v1::files::*, api::v1::users::*, views::index::*};
use tera::Tera;

lazy_static::lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera
    };
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub database: Database,
    pub storage: Storage,
    pub tera: Tera,
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

    //? This is very hacky, but it works for now.
    let storage;
    if config.storage.local.enabled {
        info!("Using local storage module");
        storage = Storage::Local(config.storage.local.path.clone());
    } else {
        info!("Using S3 storage module");
        storage = Storage::S3(config.storage.s3.clone());
    }

    let state = AppState {
        config,
        database,
        storage,
        tera: TEMPLATES.clone(),
    };

    info!(
        "Starting server on http://{}:{} ...",
        _config.server.host, _config.server.port
    );

    HttpServer::new(move || App::new().app_data(state.clone()).configure(routes))
        .bind(format!("{}:{}", _config.server.host, _config.server.port))
        .unwrap()
        .run()
        .await
        .unwrap();
}
