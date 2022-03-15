use actix_web::{web, App, HttpServer};
use mongodb::{Database};

mod log;
mod routes;
mod config;
mod crypto;
mod startup;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    database: Database,
    config: Config,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let appstate: AppState = startup::startup().await;
    let _appstate = appstate.clone();
    if let Ok(o) = HttpServer::new(move || {
            App::new().app_data(web::Data::new(_appstate.clone())).service(routes::v1::files::get_file).service(routes::v1::files::delete_file).service(
                web::scope("/api").service(
                    web::scope("/v1").service(
                        web::scope("/users").route("", web::post().to(routes::v1::users::create)).route("", web::delete().to(routes::v1::users::delete)),
                    ).service(
                        web::scope("/files").route("", web::post().to(routes::v1::files::upload)),
                    ),
                ),
            )
        }).bind((&(*appstate.config.network.bind_address.to_string().as_str()), appstate.config.network.bind_port)) {
        log::info("Server started");
        o.run().await
    } else {
                log::critical("Could not bind to address");
                std::process::exit(1);
            }
}
