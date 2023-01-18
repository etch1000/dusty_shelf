pub mod dusty_b;
use diesel::Connection;
use diesel_migrations::embed_migrations;
use rocket_okapi::{
    gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};

// #![cfg(feature = "mock-database")]
// #[cfg(not(feature = "mock-database"))]
#[rocket_sync_db_pools::database("postgres")]
// #[cfg(not(feature = "mock-database"))]
pub struct DustyShelfDB(diesel::PgConnection);

#[cfg(feature = "mock-database")]
pub use dusty_b::DustyB;

impl<'r> OpenApiFromRequest<'r> for DustyShelfDB {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

#[cfg(test)]
pub fn get_db_connection() -> diesel::PgConnection {
    // #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    diesel::PgConnection::establish(&database_url).expect("write better migrations!")
}

embed_migrations!("../migrations");

pub fn migrate(database_url: &str) {
    let conn = diesel::PgConnection::establish(database_url).expect("Database not working");

    embedded_migrations::run(&conn).expect("write better migrations!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn connect_to_db() {
        let conn = get_db_connection();

        conn.begin_test_transaction().unwrap();

        embedded_migrations::run(&conn).expect("write better migrations!")
    }
}
