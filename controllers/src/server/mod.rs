pub mod catchers;
pub mod config;
pub mod ds_errors;
pub mod launch_failures;

use crate::{dusty_b, dusty_shelf};
use rocket::time::Duration;
use rocket::{figment::Figment, shield, Build, Ignite, Rocket};
use rocket_okapi::{
    mount_endpoints_and_merged_docs,
    rapidoc::{make_rapidoc, RapiDocConfig},
    settings::UrlObject,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use yansi::Paint;

pub fn create_dusty_service(_migration_url: String, config: Figment) -> Rocket<Build> {
    use catchers::*;

    let shield = shield::Shield::default()
        .enable(shield::XssFilter::EnableBlock)
        .enable(shield::NoSniff::Enable)
        .enable(shield::Frame::SameOrigin)
        .enable(shield::Hsts::Enable(Duration::weeks(3)))
        .enable(shield::Referrer::NoReferrer);

    let mut dusty_rocket = rocket::custom(config)
        .mount("/", rocket::routes![api_home])
        .mount("/v1", make_rapidoc(&get_rapidoc_config()))
        .mount("/v1/swagger", make_swagger_ui(&get_swagger_config()))
        .attach(dusty_b::DustyShelfDB::fairing())
        .attach(shield)
        .register("/", rocket::catchers![not_found, unauthorized])
        .manage(dtos::ConfigDto {
            name: String::from("etch1000"),
            age: 25,
        });

    let openapi_settings = rocket_okapi::settings::OpenApiSettings::default();

    let custom_route_specification = (vec![], custom_openapi_specification());

    mount_endpoints_and_merged_docs! {
        dusty_rocket, "/v1".to_owned(), openapi_settings,
        "/" => custom_route_specification,
        "/dusty_shelf" => dusty_shelf::get_index_route_and_doc(&openapi_settings),
    };

    dusty_rocket
}

#[rocket::get("/")]
async fn api_home() -> rocket::response::Redirect {
    rocket::response::Redirect::to(rocket::uri!("/v1"))
}

pub fn start_dusty_server(db_migration_url: String, config: Figment) {
    log::info!("Starting Dusty Server");

    if !config
        .find_value("cli_colors")
        .unwrap_or_else(|_| rocket::figment::value::Value::from(false))
        .to_bool()
        .unwrap_or(false)
    {
        Paint::disable();
    }

    let launch_rocket_blackbox = rocket::execute(launch_rocket(db_migration_url.clone(), config));

    println!("{launch_rocket_blackbox:#?}");

    match launch_rocket_blackbox {
        Ok(_service) => log::info!("Rocket shut down gracefully"),
        Err(err) => launch_failures::launch_failure_handler(err),
    }
}

async fn launch_rocket(
    db_migration_url: String,
    config: Figment,
) -> Result<Rocket<Ignite>, rocket::Error> {
    create_dusty_service(db_migration_url.clone(), config)
        .launch()
        .await
}

// Swagger
fn get_swagger_config() -> SwaggerUIConfig {
    SwaggerUIConfig {
        urls: vec![UrlObject::new("Dusty Shelf", "/v1/openapi.json")],
        deep_linking: true,
        display_request_duration: true,
        ..Default::default()
    }
}

fn get_rapidoc_config() -> RapiDocConfig {
    use rocket_okapi::rapidoc::{GeneralConfig, HideShowConfig, SlotsConfig, UiConfig};

    RapiDocConfig {
        title: Some("Dusty Shelf API documentation | Rapidoc".to_owned()),
        ui: UiConfig {
            #[cfg(debug_assertions)]
            theme: rocket_okapi::rapidoc::Theme::Dark,
            ..Default::default()
        },
        general: GeneralConfig {
            spec_urls: vec![UrlObject::new("Main", "/v1/openapi.json")],
            sort_tags: true,
            ..Default::default()
        },
        hide_show: HideShowConfig {
            allow_spec_url_load: false,
            allow_spec_file_load: false,
            allow_server_selection: true,
            allow_authentication: true,
            ..Default::default()
        },
        slots: SlotsConfig {
            logo: Some("https://rustacean.net/assets/rustacean-orig-noshadow.svg".to_owned()),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn custom_openapi_specification() -> okapi::openapi3::OpenApi {
    use okapi::openapi3::*;

    OpenApi {
        openapi: OpenApi::default_version(),
        info: Info {
            title: "Dusty Shelf API".to_owned(),
            description: Some(
                r#"
This is a private API that is not directly accessible.
            "#
                .to_owned(),
            ),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            ..Default::default()
        },
        ..Default::default()
    }
}
