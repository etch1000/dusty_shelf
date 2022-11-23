use diesel::{Insertable, Queryable};
use crate::schema::*;
use serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;

#[derive(Deserialize)]
pub struct Config {
    pub name: String,
    pub age: u8,
}

#[derive(Deserialize, Serialize, Queryable, Debug, Insertable, JsonSchema)]
#[table_name = "books"]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub description: String,
    pub published: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct UpdateResponse {
    pub response: String,
}
