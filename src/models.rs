use crate::scheme::*;

use diesel::{Insertable, Queryable};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use rocket_sync_db_pools::{database, diesel::PgConnection};

#[derive(Deserialize)]
pub struct Config {
    pub name: String,
    pub age: u8,
}

#[derive(Deserialize, Serialize, Queryable, Debug, Insertable, JsonSchema, PartialEq)]
#[diesel(table_name = books)]
#[serde(crate = "rocket::serde")]
pub struct Book {
    pub id: i32,
    #[validate(length(min = 1))]
    pub title: String,
    #[validate(length(min = 1))]
    pub author: String,
    #[validate(length(min = 1))]
    pub description: String,
    pub published: bool,
    pub encoded: Vec<u8>,
}

impl AsRef<[u8]> for Book {
    fn as_ref(&self) -> &[u8] {
        &self.encoded
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema, Queryable)]
pub struct DSResponse {
    pub response: String,
}

#[database("postgres")]
pub struct Db(PgConnection);

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct DSError {
    pub err: String,
    pub msg: Option<String>,
    pub code: u16,
}

impl DSError {
    pub fn default_401() -> Json<DSError> {
        Json(DSError {
            err: String::from("Unauthorized"),
            msg: Some(String::from(
                "You are not authorized to perform this action",
            )),
            code: 401,
        })
    }

    pub fn default_404() -> Json<DSError> {
        Json(DSError {
            err: String::from("Not Found"),
            msg: Some(String::from("There's just Dust all over here")),
            code: 404,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DSUser {
    pub aud: String,
    pub sub: String,
    pub exp: u128,
}
