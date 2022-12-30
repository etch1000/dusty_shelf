use crate::ds_errors::unauthorized_response;
use dotenv::dotenv;
use okapi::openapi3::{Object, Responses, SecurityRequirement, SecurityScheme, SecuritySchemeData};
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{FromRequest, Request},
};
use rocket_okapi::{
    gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DSUser(dtos::DSUserDto);

impl<T> PartialEq<T> for DSUser
where
    T: dtos::DSUserLike,
{
    fn eq(&self, other: &T) -> bool {
        self.id == other.id()
    }
}

lazy_static::lazy_static! {
    static ref DECODEKEY: jsonwebtoken::DecodingKey = {
        dotenv().ok();

        let secret = std::env::var("JWT_SECRET").expect("`JWT_SECRET` must be set in environment");

        jsonwebtoken::DecodingKey::from_secret(secret.as_bytes())
    };
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DSUser {
    type Error = Status;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, (Self::Error, Self::Error), ()> {
        let header = match req.headers().get_one("Authorization") {
            Some(header) => header,
            None => return Outcome::Failure((Status::Unauthorized, Status::Unauthorized)),
        };

        let result = if let Some(header) = header.strip_prefix("Bearer ") {
            decode_from_jwt(header, &DECODEKEY)
        } else {
            Err(Status::Unauthorized)
        };

        match result {
            Ok(ds_user) => Outcome::Success(ds_user),
            Err(_status) => Outcome::Failure((Status::Unauthorized, Status::Unauthorized)),
        }
    }
}

impl<'r> OpenApiFromRequest<'r> for DSUser {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let security_scheme = SecurityScheme {
            description: Some("Requires a Bearer Token to Access.".to_owned()),
            data: SecuritySchemeData::ApiKey {
                name: "Authorization".to_owned(),
                location: "header".to_owned(),
            },
            extensions: Object::default(),
        };

        let mut security_req = SecurityRequirement::new();
        security_req.insert("DSUser".to_owned(), Vec::new());
        Ok(RequestHeaderInput::Security(
            "DSUser".to_owned(),
            security_scheme,
            security_req,
        ))
    }

    fn get_responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        use okapi::openapi3::RefOr;

        Ok(Responses {
            responses: okapi::map! {
                "401".to_owned() => RefOr::Object(unauthorized_response(gen))
            },
            ..Default::default()
        })
    }
}

impl std::ops::Deref for DSUser {
    type Target = dtos::DSUserDto;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn decode_from_jwt<T: serde::de::DeserializeOwned>(
    bearer: &str,
    secret: &jsonwebtoken::DecodingKey,
) -> Result<T, Status> {
    use jsonwebtoken as jwt;

    jwt::decode(bearer, secret, &jwt::Validation::default())
        .map_err(|_| Status::Unauthorized)
        .map(|data| data.claims)
}
