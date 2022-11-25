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

#[derive(Deserialize, Serialize, Queryable, Debug, Insertable, JsonSchema)]
#[diesel(table_name = books)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub description: String,
    pub published: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct DSResponse {
    pub response: String,
}

#[database("dustyshelf")]
pub struct Db(PgConnection);
