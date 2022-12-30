use dotenv::dotenv;
use dtos::DSUserDto;
use jsonwebtoken as jwt;
use rocket::http::Status;

lazy_static::lazy_static! {
    static ref ENCODEKEY: jwt::EncodingKey = {
        dotenv().ok();

        let secret = std::env::var("JWT_SECRET").expect("`JWT_SECRET` must be set in environment");

        jwt::EncodingKey::from_secret(secret.as_bytes())
    };

    static ref DECODEKEY: jwt::DecodingKey = {
        dotenv().ok();

        let secret = std::env::var("JWT_SECRET").expect("`JWT_SECRET` must be set in environment");

        jwt::DecodingKey::from_secret(secret.as_bytes())
    };
}

pub fn _create_jwt(ds_user: DSUserDto) -> Result<String, jwt::errors::Error> {
    let ds_user_access = DSUserDto {
        id: ds_user.id,
        aud: ds_user.aud,
        sub: ds_user.sub,
        exp: ds_user.exp,
    };

    jwt::encode(&jwt::Header::default(), &ds_user_access, &ENCODEKEY)
}

// pub fn parse_jwt(jwtoken: &str) -> Result<DSUserDto, Json<DSError>> {
//     let ds_user: DSUserDto = jwt::decode(jwtoken, &DECODEKEY, &jwt::Validation::default())
//         .map_err(|_| {
//             Json(DSError {
//                 err: String::from("Failed to Parse JWT"),
//                 msg: Some(String::from("Failed to Parse JWT")),
//                 code: 400,
//             })
//         })?
//         .claims;

//     Ok(ds_user)
// }

pub fn decode_jwt<T: serde::de::DeserializeOwned>(
    jwtoken: &str,
    jwt_secret: &jwt::DecodingKey,
) -> Result<DSUserDto, Status> {
    jwt::decode(jwtoken, jwt_secret, &jwt::Validation::default())
        .map_err(|_| Status::Unauthorized)
        .map(|data| data.claims)
}
