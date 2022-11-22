#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

use diesel::{prelude::*, table, Insertable, Queryable};
use rocket::{fairing::AdHoc, response::Debug, serde::json::Json, State};
use rocket_okapi::{
    openapi, openapi_get_routes,
    request::{OpenApiFromRequest, RequestHeaderInput},
    settings::UrlObject,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
    JsonSchema,
};
use rocket_sync_db_pools::{database, diesel::PgConnection};
use serde::{Deserialize, Serialize};

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

table! {
    books (id) {
        id -> Int4,
        title -> Varchar,
        author -> Varchar,
        description -> Text,
        published -> Bool,
    }
}

#[database("rootkill")]
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

#[derive(Deserialize)]
struct Config {
    name: String,
    age: u8,
}

#[derive(Deserialize, Serialize, Queryable, Debug, Insertable, JsonSchema)]
#[table_name = "books"]
struct Book {
    id: i32,
    title: String,
    author: String,
    description: String,
    published: bool,
}

#[derive(Serialize, JsonSchema)]
struct UpdateResponse {
    response: String,
}

#[openapi]
#[get("/")]
fn index() -> &'static str {
    "Welcome to Dusty Shelf"
}

#[openapi]
#[get("/config")]
fn get_config(config: &State<Config>) -> String {
    format!("Hello {}, welcome to the club {}!", config.name, config.age)
}

#[openapi]
#[get("/random")]
fn random_book() -> Json<Book> {
    Json(Book {
        id: 1,
        title: String::from("Your Personal Diary"),
        author: String::from("You"),
        description: String::from("You know what this is about! We don't want to know! :)"),
        published: true,
    })
}

#[openapi]
#[get("/<id>")]
async fn get_by_id(connection: Db, id: i32) -> Json<Book> {
    connection
        .run(move |c| books::table.filter(books::id.eq(&id)).first(c))
        .await
        .map(Json)
        .expect(format!("Cannot find book with id : {}", id).as_str())
}

#[openapi]
#[get("/all")]
async fn get_all_books(connection: Db) -> Json<Vec<Book>> {
    connection
        .run(|c| books::table.load(c))
        .await
        .map(Json)
        .expect("Failed to fetch all books for you! :(")
}

#[openapi]
#[post("/", data = "<book>")]
async fn add_book(connection: Db, book: Json<Book>) -> Json<Book> {
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

#[openapi]
#[delete("/<id>")]
async fn delete_book(connection: Db, id: i32) -> Result<Option<()>> {
    let res = connection
        .run(move |c| {
            diesel::delete(books::table)
                .filter(books::id.eq(id))
                .execute(c)
        })
        .await?;

    Ok((res == 1).then(|| ()))
}

#[openapi]
#[put("/book/<id>", data = "<book>")]
async fn update_book(connection: Db, id: i32, book: Json<Book>) -> Json<UpdateResponse> {
    match connection
        .run(move |c| {
            diesel::update(books::table.filter(books::id.eq(id)))
                .set((
                    books::title.eq(&book.title),
                    books::description.eq(&book.description),
                ))
                .execute(c)
        })
        .await
    {
        Ok(res) => Json(UpdateResponse {
            response: format!("Book Successfully Updated! RESULT: {}", res),
        }),
        Err(e) => Json(UpdateResponse {
            response: format!("Failed to update the book! REASON: {}", e),
        }),
    }
}

// Swagger
fn get_swagger_config() -> SwaggerUIConfig {
    SwaggerUIConfig {
        urls: vec![
            UrlObject::new("Home", "/openapi.json"),
            UrlObject::new("Add Book", "/add_book/openapi.json"),
            UrlObject::new("Update Book", "/update/openapi.json"),
            UrlObject::new("Delete Book", "/delete/openapi.json"),
        ],
        ..Default::default()
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::fairing())
        .attach(AdHoc::config::<Config>())
        .mount("/", openapi_get_routes![index, get_config])
        .mount("/swagger", make_swagger_ui(&get_swagger_config()))
        .mount(
            "/book",
            openapi_get_routes![random_book, get_by_id, get_all_books],
        )
        .mount("/add_book", openapi_get_routes![add_book])
        .mount("/delete", openapi_get_routes![delete_book])
        .mount("/update", openapi_get_routes![update_book])
}
