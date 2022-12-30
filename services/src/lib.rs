#[allow(unused_imports)]
#[macro_use]
extern crate diesel_migrations;

pub mod db_conn;
pub mod services;

pub use diesel_migrations::EmbedMigrations;
pub use services::*;
