use crate::scheme::*;
use diesel::{Insertable, Queryable};
use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::{
    request::{OpenApiFromRequest, RequestHeaderInput},
    JsonSchema,
};
use rocket_sync_db_pools::{database, diesel::PgConnection};

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

#[database("postgres")]
pub struct Db(PgConnection);

impl<'r> OpenApiFromRequest<'r> for Db {
    fn from_request_input(
        _gen: &mut rocket_okapi::gen::OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<rocket_okapi::request::RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct DSUser {
    pub id: i32,
    pub aud: String,
    pub sub: String,
    pub exp: u128,
}

impl DSUser {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_aud(&self) -> String {
        format!("{}", self.aud)
    }

    pub fn get_sub(&self) -> String {
        format!("{}", self.sub)
    }

    pub fn get_exp(&self) -> u128 {
        self.exp
    }
}

pub trait DSUserLike {
    fn id(&self) -> i32;

    fn aud(&self) -> String;

    fn sub(&self) -> String;

    fn exp(&self) -> u128;
}

impl DSUserLike for DSUser {
    fn id(&self) -> i32 {
        self.id
    }

    fn aud(&self) -> String {
        self.aud.clone()
    }

    fn sub(&self) -> String {
        self.sub.clone()
    }

    fn exp(&self) -> u128 {
        self.exp
    }
}

impl PartialEq for DSUser {
    fn eq(&self, other: &DSUser) -> bool {
        self.id == other.id()
            && self.aud == other.aud()
            && self.sub == other.sub()
            && self.exp == other.exp()
    }

    fn ne(&self, other: &DSUser) -> bool {
        self.id != other.id()
            || self.aud != other.aud()
            || self.sub != other.sub()
            || self.exp != other.exp()
    }
}

impl Eq for DSUser {}
