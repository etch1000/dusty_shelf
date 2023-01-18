use rocket::{
    config::{Config, Ident, LogLevel, Shutdown},
    data::{ByteUnit, Limits},
    figment::{Figment, Profile},
    serde::json::Value,
};
use serde_json::Map;
use std::{net::IpAddr, str::FromStr};

pub fn get_config() -> Figment {
    dotenv::dotenv().ok();

    let url = std::env::var("DATABASE_URL").expect("`DATABASE_URL` mut be set");
    println!("{url:?}");

    let mut database_config = Map::new();

    let mut databases = Map::new();

    database_config.insert("url".to_owned(), Value::from(url.as_str()));

    if cfg!(test) || cfg!(feature = "mock-database") {
        database_config.insert("pool_size".to_owned(), Value::from(1i32));
    } else {
        database_config.insert("pool_size".to_owned(), Value::from(32i32));
    }

    databases.insert("postgres".to_owned(), Value::from(database_config));

    let address =
        IpAddr::from_str(&std::env::var("HOST_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_owned()))
            .expect("Could not parse `HOST_ADDRESS`. (only IPv4 or IPv6 allowed)");

    let port = std::env::var("PORT_NUMBER")
        .unwrap_or_else(|_| "8080".to_owned())
        .parse::<u16>()
        .unwrap();

    let keep_alive = 0;

    let server_identifier = Ident::none();

    let limits = Limits::default().limit("json", ByteUnit::Byte(102400));

    let profile = get_profile();

    let config = match profile.as_ref() {
        "debug" => Config {
            profile,
            address,
            port,
            workers: 10,
            keep_alive,
            limits,
            ident: server_identifier,
            temp_dir: std::env::temp_dir().into(),
            log_level: LogLevel::Normal,
            shutdown: Shutdown {
                ctrlc: true,
                #[cfg(unix)]
                signals: {
                    let mut set = std::collections::HashSet::new();
                    set.insert(rocket::config::Sig::Term);
                    set.insert(rocket::config::Sig::Hup);
                    set
                },
                grace: 1,
                mercy: 1,
                force: true,
                ..Default::default()
            },
            cli_colors: true,
            ..Default::default()
        },
        "release" | _ => Config {
            profile,
            address,
            port,
            workers: 10,
            keep_alive,
            limits,
            ident: server_identifier,
            temp_dir: std::env::temp_dir().into(),
            log_level: LogLevel::Critical,
            shutdown: Shutdown::default(),
            cli_colors: false,
            ..Default::default()
        },
    };

    Figment::from(config)
}

fn get_profile() -> Profile {
    match std::env::var("ROCKET_PROFILE").as_ref().map(String::as_str) {
        Ok("debug") => Profile::new("debug"),
        Ok("release") => Profile::new("release"),
        Ok(string) => {
            panic!(
                "Expected profile `{}` set using `ROCKET_PROFILE`, use `debug` or `release`.",
                string
            );
        }
        Err(_) => {
            panic!("No `ROCKET_PROFILE` provided.");
        }
    }
}
