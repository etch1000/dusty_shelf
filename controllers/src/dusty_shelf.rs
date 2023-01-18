use crate::auth::DSUser;
use diesel::prelude::*;
use dtos::{ConfigDto, DSErrorDto, DSResponseDto};
use models::{books, Book, Db};
use okapi::openapi3::OpenApi;
use rocket::{get, response::status, serde::json::Json, Route, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};

pub fn get_index_route_and_doc(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: index,
        get_config,
        get_random_book,
        get_by_id,
        get_all_books,
        add_book,
        delete_book,
        update_book
    ]
}

#[openapi(tag = "Home")]
#[get("/")]
fn index(_ds_user: DSUser) -> &'static str {
    "Welcome To The Dusty Shelf"
}

#[openapi(tag = "ConfigDto")]
#[get("/config")]
fn get_config(_ds_user: DSUser, config: &State<ConfigDto>) -> String {
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
) -> Result<Json<Book>, status::NotFound<Json<DSErrorDto>>> {
    match connection
        .run(move |c| books::table.filter(books::id.eq(&id)).get_result(c))
        .await
        .map(Json)
    {
        Ok(book) => Ok(book),
        Err(_) => Err(status::NotFound(DSErrorDto::default_404())),
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
) -> Result<Json<DSResponseDto>, status::NotFound<Json<DSErrorDto>>> {
    let res = connection
        .run(move |c| {
            diesel::delete(books::table)
                .filter(books::id.eq(id))
                .execute(c)
        })
        .await;

    if res == Ok(1) {
        Ok(Json(DSResponseDto {
            response: format!("Book with id: {} is removed from the Shelf now", id),
        }))
    } else {
        Err(status::NotFound(DSErrorDto::default_404()))
    }
}

#[openapi(tag = "Update Book")]
#[put("/update_book/<id>", data = "<book>")]
async fn update_book(
    _ds_user: DSUser,
    connection: Db,
    id: i32,
    book: Json<Book>,
) -> Result<Json<DSResponseDto>, status::NotFound<Json<DSErrorDto>>> {
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
        Ok(res) => Ok(Json(DSResponseDto {
            response: format!("Book Updated Successfully! RESULT: {}", res),
        })),
        Err(_) => Err(status::NotFound(DSErrorDto::default_404())),
    }
}
