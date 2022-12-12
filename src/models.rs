use crate::schema::*;
use diesel::{Insertable, Queryable};
use rocket_okapi::JsonSchema;
use rocket_sync_db_pools::{database, diesel::PgConnection};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Config {
    pub name: String,
    pub age: u8,
}

#[derive(Deserialize, Serialize, Queryable, Debug, Insertable, JsonSchema, PartialEq)]
#[diesel(table_name = books)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub description: String,
    pub published: bool,
}

#[derive(Serialize, JsonSchema, Queryable)]
pub struct DSResponse {
    pub response: String,
}

#[database("postgres")]
pub struct Db(PgConnection);

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Error {
    pub err: String,
    pub msg: Option<String>,
    pub code: u16,
}
