#![recursion_limit = "256"]

pub mod models;
pub mod schema;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use models::*;
use rocket::{fairing::AdHoc, response::Debug, serde::json::Json, State};
use rocket_okapi::{
    openapi, openapi_get_routes,
    request::{OpenApiFromRequest, RequestHeaderInput},
    settings::UrlObject,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use schema::books;

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;


impl<'r> OpenApiFromRequest<'r> for Db {
    fn from_request_input(
        _gen: &mut rocket_okapi::gen::OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<rocket_okapi::request::RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

#[openapi(tag = "Home")]
#[get("/")]
fn index() -> &'static str {
    "Welcome To The Dusty Shelf"
}

#[openapi(tag = "Config")]
#[get("/config")]
fn get_config(config: &State<Config>) -> String {
    format!("Hello {}, welcome to the club {}!", config.name, config.age)
}

#[openapi(tag = "Books")]
#[get("/book/random")]
fn get_random_book() -> Json<Book> {
    Json(Book {
        id: 0,
        title: String::from("Your Personal Diary"),
        author: String::from("You"),
        description: String::from("You know what this is about! We don't want to know! :)"),
        published: true,
    })
}

#[openapi(tag = "Books")]
#[get("/book/<id>")]
async fn get_by_id(connection: Db, id: i32) -> Json<Book> {
    connection
        .run(move |c| books::table.filter(books::id.eq(&id)).get_result(c))
        .await
        .map(Json)
        .unwrap_or_else(|_| panic!("Cannot find book with id : {}", id))
}

#[openapi(tag = "Books")]
#[get("/book/all")]
async fn get_all_books(connection: Db) -> Json<Vec<Book>> {
    connection
        .run(|c| books::table.load(c))
        .await
        .map(Json)
        .expect("Failed to fetch all books for you! :(")
}

#[openapi(tag = "Add Book")]
#[post("/add_book", data = "<book>")]
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

#[openapi(tag = "Delete Book")]
#[delete("/delete_book/<id>")]
async fn delete_book(connection: Db, id: i32) -> Result<Option<()>> {
    let res = connection
        .run(move |c| {
            diesel::delete(books::table)
                .filter(books::id.eq(id))
                .execute(c)
        })
        .await?;

    Ok((res == 1).then_some(()))
}

#[openapi(tag = "Update Book")]
#[put("/update_book/<id>", data = "<book>")]
async fn update_book(connection: Db, id: i32, book: Json<Book>) -> Json<DSResponse> {
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
        Ok(res) => Json(DSResponse {
            response: format!("Book Successfully Updated! RESULT: {}", res),
        }),
        Err(e) => Json(DSResponse {
            response: format!("Failed to update the book! REASON: {}", e),
        }),
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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::fairing())
        .attach(AdHoc::config::<Config>())
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
