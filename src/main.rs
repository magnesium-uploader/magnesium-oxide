use actix_web::{web, App, HttpServer};
use mongodb::{options::ClientOptions, Client, Database};

mod log;
mod routes;
mod config;
mod crypter;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    database: Database,
    config: Config,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Parse a connection string into an options struct.
    log::info("Starting server...");
    let config = match Config::new("config.toml") {
        Ok(config) => config,
        Err(_) => {
            log::critical("Could not parse config.toml");
            std::process::exit(1);
        }
    };

    let client_options = match ClientOptions::parse(&config.mongo.uri).await {
        Ok(client_options) => client_options,
        Err(_) => {
            log::critical("Could not parse mongo uri");
            std::process::exit(1);
        }
    };

    let client = match Client::with_options(client_options) {
        Ok(client) => client,
        Err(_) => {
            log::critical("Could not connect to mongo");
            std::process::exit(1);
        }
    };

    let _database = client.database(&config.mongo.database);
    let appstate = AppState {
        database: _database,
        config: config.clone(),
    };

    match HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(appstate.clone()))
            .service(routes::v1::files::get_file)
            .service(routes::v1::files::delete_file)
            .service(
                web::scope("/api").service(
                    web::scope("/v1")
                        .service(
                            web::scope("/users")
                                .route("", web::post().to(routes::v1::users::create))
                                .route("", web::delete().to(routes::v1::users::delete)),
                        )
                        .service(
                            web::scope("/files")
                                .route("", web::post().to(routes::v1::files::upload)),
                        ),
                ),
            )
    })
    .bind((config.network.bind_address.to_string().as_str().clone(), config.network.bind_port.clone()))
    {
        Ok(o) => {
            log::info("Server started");
            o.run().await
        }
        Err(_) => {
            log::critical("Could not bind to address");
            std::process::exit(1);
        }
    }
}
