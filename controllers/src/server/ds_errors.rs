use okapi::openapi3::{MediaType, Response as OpenApiResponse};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket_okapi::{gen::OpenApiGenerator, JsonSchema};

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

pub fn unauthorized_response(gen: &mut OpenApiGenerator) -> OpenApiResponse {
    let schema = gen.json_schema::<DSError>();

    OpenApiResponse {
        description: "\
            # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
            This response is given when you request a page that you don't have access to \
            or you have not provided any authentication. "
            .to_owned(),
        content: okapi::map! {
            "application/json".to_owned() => MediaType {
                schema: Some(schema),
                ..Default::default()
            }
        },
        ..Default::default()
    }
}
