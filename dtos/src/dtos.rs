use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket_okapi::JsonSchema;

#[derive(Deserialize)]
pub struct ConfigDto {
    pub name: String,
    pub age: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DSUserDto {
    pub id: i32,
    pub aud: String,
    pub sub: String,
    pub exp: u128,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct DSResponseDto {
    pub response: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct DSErrorDto {
    pub err: String,
    pub msg: Option<String>,
    pub code: u16,
}

pub trait DSUserLike {
    fn id(&self) -> i32;

    fn aud(&self) -> String;

    fn sub(&self) -> String;

    fn exp(&self) -> u128;
}

impl DSUserLike for DSUserDto {
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

impl DSUserDto {
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

impl DSErrorDto {
    pub fn default_401() -> Json<DSErrorDto> {
        Json(DSErrorDto {
            err: String::from("Unauthorized"),
            msg: Some(String::from(
                "You are not authorized to perform this action",
            )),
            code: 401,
        })
    }

    pub fn default_404() -> Json<DSErrorDto> {
        Json(DSErrorDto {
            err: String::from("Not Found"),
            msg: Some(String::from("There's just Dust all over here")),
            code: 404,
        })
    }
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JWTokenDto {
    pub jwtoken: String,
    pub refresh: String,
}
