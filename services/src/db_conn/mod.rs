#![cfg(test)]

use chrono::{DateTime, Duration, Utc};
use diesel::{Connection, PgConnection};
use diesel_migrations::embed_migrations;

#[allow(dead_code)]
pub(crate) fn db_connection() -> PgConnection {
    // #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("`DATABASE_URL` not set in env");

    PgConnection::establish(&database_url).expect("Database not working")
}

embed_migrations!("../migrations");

#[allow(dead_code)]
pub(crate) fn run_migrations(database_url: &str) {
    let conn = PgConnection::establish(&database_url).expect("Database not working");

    embedded_migrations::run(&conn).expect("Write better migrations");
}

#[allow(dead_code)]
pub(crate) fn assert_about_now(date: DateTime<Utc>) {
    let now = Utc::now();
    assert!((now - date) < max_time_difference());
}

#[allow(dead_code)]
pub(crate) fn assert_utc_datetime(datetime_1: DateTime<Utc>, date_str: &str) {
    let datetime = DateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S.f%#z").unwrap();

    let datetime_utc_2 = datetime.with_timezone(&Utc);

    assert_eq!(datetime_1, datetime_utc_2);
}

#[allow(dead_code)]
pub(crate) fn string_to_utc_datetime(date_str: &str) -> DateTime<Utc> {
    let datetime = DateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S%.f%#z").unwrap();

    datetime.with_timezone(&Utc)
}

#[allow(dead_code)]
pub(crate) fn max_time_difference() -> Duration {
    Duration::minutes(10)
}
