pub mod models;
pub mod scheme;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use dotenv::dotenv;
use jsonwebtoken as jwt;
use models::*;
use okapi::openapi3::{
    MediaType, Object, Response as OpenApiResponse, Responses, SecurityRequirement, SecurityScheme,
    SecuritySchemeData,
};
use rocket::{
    fairing::AdHoc,
    http,
    http::Status,
    local::blocking::Client,
    outcome::Outcome,
    request::{FromRequest, Request},
    response::{status, Debug},
    serde::json::Json,
    State,
};
use rocket_okapi::{
    gen::OpenApiGenerator,
    openapi, openapi_get_routes,
    request::{OpenApiFromRequest, RequestHeaderInput},
    settings::UrlObject,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use scheme::books;

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

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

    static ref JWT_SECRET: jwt::DecodingKey = {
        dotenv().ok();

        let secret = std::env::var("JWT_SECRET").expect("`JWT_SECRET` must be set in environment");

        jwt::DecodingKey::from_secret(secret.as_bytes())
    };
}

fn _create_jwt(ds_user: DSUser) -> Result<String, jwt::errors::Error> {
    let ds_user_access = DSUser {
        aud: ds_user.aud,
        sub: ds_user.sub,
        exp: ds_user.exp,
    };

    jwt::encode(&jwt::Header::default(), &ds_user_access, &ENCODEKEY)
}

// fn parse_jwt(jwtoken: &str) -> Result<DSUser, Json<DSError>> {
//     let ds_user: DSUser = jwt::decode(jwtoken, &DECODEKEY, &jwt::Validation::default())
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

fn decode_jwt<T: serde::de::DeserializeOwned>(
    jwtoken: &str,
    jwt_secret: &jwt::DecodingKey,
) -> Result<DSUser, Status> {
    jwt::decode(jwtoken, jwt_secret, &jwt::Validation::default())
        .map_err(|_| Status::Unauthorized)
        .map(|data| data.claims)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DSUser {
    type Error = http::Status;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, (Self::Error, Self::Error), ()> {
        let header = match req.headers().get_one("Authorization") {
            Some(header) => header,
            None => return Outcome::Failure((Status::Unauthorized, Status::Unauthorized)),
        };

        let result = if let Some(header) = header.strip_prefix("Bearer ") {
            decode_jwt::<DSUser>(header, &JWT_SECRET)
        } else {
            decode_jwt::<DSUser>(header, &JWT_SECRET)
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

    fn get_responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        use okapi::openapi3::RefOr;

        Ok(Responses {
            responses: okapi::map! {
                "401".to_owned() => RefOr::Object(unauthorized_response(_gen))
            },
            ..Default::default()
        })
    }
}

impl<'r> OpenApiFromRequest<'r> for Db {
    fn from_request_input(
        _gen: &mut rocket_okapi::gen::OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<rocket_okapi::request::RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

#[catch(404)]
fn not_found() -> Json<DSError> {
    DSError::default_404()
}

#[catch(401)]
fn unauthorized() -> Json<DSError> {
    DSError::default_401()
}

fn unauthorized_response(gen: &mut OpenApiGenerator) -> OpenApiResponse {
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

#[openapi(tag = "Home")]
#[get("/")]
fn index(_ds_user: DSUser) -> &'static str {
    "Welcome To The Dusty Shelf"
}

#[openapi(tag = "Config")]
#[get("/config")]
fn get_config(_ds_user: DSUser, config: &State<Config>) -> String {
    format!("Hello {}, welcome to the club {}!", config.name, config.age)
}

#[openapi(tag = "Books")]
#[get("/book/random")]
fn get_random_book(_ds_user: DSUser) -> Json<Book> {
    Json(Book {
        id: 0,
        title: String::from("Your Personal Diary"),
        author: String::from("You"),
        description: String::from("You know what this is about! We don't want to know! :)"),
        published: true,
        encoded: vec![0],
    })
}

#[openapi(tag = "Books")]
#[get("/book/<id>")]
async fn get_by_id(
    _ds_user: DSUser,
    connection: Db,
    id: i32,
) -> Result<Json<Book>, status::NotFound<Json<DSError>>> {
    match connection
        .run(move |c| books::table.filter(books::id.eq(&id)).get_result(c))
        .await
        .map(Json)
    {
        Ok(book) => Ok(book),
        Err(_) => Err(status::NotFound(DSError::default_404())),
    }
}

#[openapi(tag = "Books")]
#[get("/book/all")]
async fn get_all_books(_ds_user: DSUser, connection: Db) -> Json<Vec<Book>> {
    connection
        .run(|c| books::table.load(c))
        .await
        .map(Json)
        .expect("Failed to fetch all books for you! :(")
}

#[openapi(tag = "Add Book")]
#[post("/add_book", format = "json", data = "<book>")]
async fn add_book(_ds_user: DSUser, connection: Db, book: Json<Book>) -> Json<Book> {
    connection
        .run(move |c| {
            diesel::insert_into(books::table)
                .values(&book.into_inner())
                .get_result(c)
        })
        .await
        .map(Json)
        .expect("Failed to put the book into Dusty Shelf")
}

#[openapi(tag = "Delete Book")]
#[delete("/delete_book/<id>")]
async fn delete_book(
    _ds_user: DSUser,
    connection: Db,
    id: i32,
) -> Result<Json<DSResponse>, status::NotFound<Json<DSError>>> {
    let res = connection
        .run(move |c| {
            diesel::delete(books::table)
                .filter(books::id.eq(id))
                .execute(c)
        })
        .await;

    if res == Ok(1) {
        Ok(Json(DSResponse {
            response: format!("Book with id: {} is removed from the Shelf now", id),
        }))
    } else {
        Err(status::NotFound(DSError::default_404()))
    }
}

#[openapi(tag = "Update Book")]
#[put("/update_book/<id>", data = "<book>")]
async fn update_book(
    _ds_user: DSUser,
    connection: Db,
    id: i32,
    book: Json<Book>,
) -> Result<Json<DSResponse>, status::NotFound<Json<DSError>>> {
    match connection
        .run(move |c| {
            diesel::update(books::table.filter(books::id.eq(id)))
                .set((
                    books::title.eq(&book.title),
                    books::author.eq(&book.author),
                    books::description.eq(&book.description),
                ))
                .execute(c)
        })
        .await
    {
        Ok(res) => Ok(Json(DSResponse {
            response: format!("Book Updated Successfully! RESULT: {}", res),
        })),
        Err(_) => Err(status::NotFound(DSError::default_404())),
    }
}

// Swagger
fn get_swagger_config() -> SwaggerUIConfig {
    SwaggerUIConfig {
        urls: vec![UrlObject::new("Dusty Shelf", "/openapi.json")],
        deep_linking: true,
        display_request_duration: true,
        ..Default::default()
    }
}

#[allow(dead_code)]
fn make_client() -> Client {
    Client::tracked(rocket()).expect("Valid Rocket Instance")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::fairing())
        .attach(AdHoc::config::<Config>())
        .register("/", catchers![not_found, unauthorized])
        .mount("/swagger", make_swagger_ui(&get_swagger_config()))
        .mount(
            "/",
            openapi_get_routes![
                index,
                get_config,
                get_random_book,
                get_by_id,
                get_all_books,
                add_book,
                delete_book,
                update_book
            ],
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{make_client, models::Book};
    use rocket::{
        http::{Header, Status},
        serde::json,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_index_status() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        let client = make_client();

        let res = client
            .get(uri!(super::index))
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!(Status::Ok, res.status());
    }

    #[test]
    fn test_index_content() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        let client = make_client();

        let res = client
            .get(uri!(super::index))
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!("Welcome To The Dusty Shelf", res.into_string().unwrap());
    }

    #[test]
    fn test_config_status() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        let client = make_client();

        let res = client
            .get(uri!(super::get_config))
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!(Status::Ok, res.status());
    }

    #[test]
    fn test_config_content() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        let client = make_client();

        let res = client
            .get(uri!(super::get_config))
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!(
            String::from("Hello etch1000, welcome to the club 24!"),
            res.into_string().unwrap()
        );
    }

    #[test]
    fn test_random_book_status() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        let client = make_client();

        let res = client
            .get(uri!(super::get_random_book))
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!(Status::Ok, res.status());
    }

    #[test]
    fn test_random_book_content() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        let client = make_client();

        let exp_res = Json(Book {
            id: 0,
            title: String::from("Your Personal Diary"),
            author: String::from("You"),
            description: String::from("You know what this is about! We don't want to know! :)"),
            published: true,
            encoded: vec![0],
        });

        let res = client
            .get(uri!(super::get_random_book))
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!(exp_res.into_inner(), res.into_json().unwrap());
    }

    #[test]
    fn test_get_by_id_status() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        let client = make_client();

        let res = client
            .get("/book/0")
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!(Status::Ok, res.status());
    }

    #[test]
    fn test_get_by_id_content() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        let client = make_client();

        let exp_res = Json(Book {
            id: 0,
            title: String::from("Personal Diary"),
            author: String::from("You"),
            description: String::from("It's a secret! Shhhh..."),
            published: true,
            encoded: vec![0],
        });

        let res = client
            .get("/book/0")
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!(exp_res.into_inner(), res.into_json().unwrap());
    }

    #[test]
    fn test_get_by_id_404_status() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        let client = make_client();

        let res = client
            .get("/book/0.0")
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!(Status::NotFound, res.status());
    }

    #[test]
    fn test_get_all_books_status() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        let client = make_client();

        let res = client
            .get(uri!(super::get_all_books))
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!(Status::Ok, res.status());
    }

    #[test]
    fn test_get_all_books_content() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        let client = make_client();

        let exp_res = Json(vec![
            Book {
                  id: 0,
                  title: String::from("Personal Diary"),
                  author: String::from("You"),
                  description: String::from("It's a secret! Shhhh..."),
                  published: true,
                  encoded: vec![0],
            },
            Book {
                  id: 1,
                  title: String::from("To Kill A Mockingbird"),
                  author: String::from("Harper Lee"),
                  description: String::from("To Kill a Mockingbird is both a young girl's coming-of-age story and a darker drama about the roots and consequences of racism and prejudice, probing how good and evil can coexist within a single community or individual."),
                  published: true,
                  encoded: vec![0],
            },
            Book {
                  id: 2,
                  title: String::from("Frankenstein"),
                  author: String::from("Mary Shelley"),
                  description: String::from("The book tells the story of Victor Frankenstein, a Swiss student of natural science who creates an artificial man from pieces of corpses and brings his creature to life."),
                  published: true,
                  encoded: vec![0],
            },
        ]);

        let res = client
            .get(uri!(super::get_all_books))
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!(exp_res.into_inner(), res.into_json::<Vec<Book>>().unwrap());
    }

    #[test]
    fn test_add_update_delete_book() {
        let test_user: DSUser = DSUser {
            aud: String::from("DUSTYSHELF"),
            sub: String::from("TESTUSER"),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + 25000,
        };

        let jwtest = _create_jwt(test_user).unwrap();

        use rocket::http::ContentType;

        let client = make_client();

        let book_to_add = Book {
            id: 10,
            title: String::from("test book"),
            author: String::from("test author"),
            description: String::from("test description"),
            published: true,
            encoded: vec![0],
        };

        let book_but_in_string = json::to_string(&book_to_add).unwrap();

        let res = client
            .post(uri!(super::add_book))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .body(&book_but_in_string)
            .dispatch();

        assert_eq!(book_but_in_string, res.into_string().unwrap());

        let update_todo = Book {
            id: 10,
            title: String::from("test book new"),
            author: String::from("test author"),
            description: String::from("test description new"),
            published: true,
            encoded: vec![0],
        };

        let update_todo_in_string = json::to_string(&update_todo).unwrap();

        let updateres = client
            .put("/update_book/10")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .body(&update_todo_in_string)
            .dispatch();

        assert_eq!(
            DSResponse {
                response: String::from("Book Updated Successfully! RESULT: 1"),
            },
            updateres.into_json::<DSResponse>().unwrap()
        );

        let delres = client
            .delete("/delete_book/10")
            .header(Header::new("Authorization", format!("Bearer {jwtest}")))
            .dispatch();

        assert_eq!(
            DSResponse {
                response: String::from("Book with id: 10 is removed from the Shelf now")
            },
            delres.into_json::<DSResponse>().unwrap()
        )
    }
}
