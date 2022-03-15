use mongodb::{options::ClientOptions, Client};

use crate::{AppState, config::Config, crypto};

pub async fn startup() -> AppState {
    println!(
"\x1b[0;31m███╗   ███╗ █████╗  ██████╗ ███╗   ██╗███████╗███████╗██╗██╗   ██╗███╗   ███╗
████╗ ████║██╔══██╗██╔════╝ ████╗  ██║██╔════╝██╔════╝██║██║   ██║████╗ ████║
██╔████╔██║███████║██║  ███╗██╔██╗ ██║█████╗  ███████╗██║██║   ██║██╔████╔██║
██║╚██╔╝██║██╔══██║██║   ██║██║╚██╗██║██╔══╝  ╚════██║██║██║   ██║██║╚██╔╝██║
██║ ╚═╝ ██║██║  ██║╚██████╔╝██║ ╚████║███████╗███████║██║╚██████╔╝██║ ╚═╝ ██║
╚═╝     ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝     ╚═╝\x1b[0m");

    println!("\x1b[0;32m[+]\x1b[0m Starting up...");

    if !std::path::Path::new("config.toml").exists() {
        Config::make_default("config.toml").unwrap();
        println!("\x1b[0;32m[+]\x1b[0m Failed to find a pre-existing configuration file, a default configuration file has been created at 'config.toml'");
        std::process::exit(1);
    }

    let config = Config::new("config.toml").unwrap();

    println!("\x1b[0;32m[+]\x1b[0m Connecting to mongo...");
    let client_options = match ClientOptions::parse(&config.mongo.uri).await {
        Ok(client_options) => client_options,
        Err(_) => {
            println!("\x1b[0;31m[-]\x1b[0m Could not connect to MongoDB");
            std::process::exit(1);
        }
    };

    let client = match Client::with_options(client_options) {
        Ok(client) => client,
        Err(_) => {
            println!("\x1b[0;31m[-]\x1b[0m Could not connect to MongoDB");
            std::process::exit(1);
        }
    };

    let _database = client.database(&config.mongo.database);

    if !config.mongo.encryption {
        println!("\x1b[0;32m[+]\x1b[0m MongoDB encryption is disabled");
    } else {
        println!("\x1b[0;32m[+]\x1b[0m MongoDB encryption is enabled");
        crypto::get_key().unwrap();
    }
    
    println!("\x1b[0;32m[+]\x1b[0m Starting server...");
    let appstate = AppState {
        database: _database,
        config: config.clone(),
    };

    println!();
    appstate
}
