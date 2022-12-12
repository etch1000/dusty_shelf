pub mod models;
pub mod schema;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use models::*;
use rocket::{fairing::AdHoc, local::blocking::Client, response::Debug, serde::json::Json, State};
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

#[catch(404)]
fn not_found() -> Json<Error> {
    Json(
        Error {
            err: String::from("Not Found"),
            msg: Some(String::from("There's just dust all over here")),
            code: 404,
        }
    )
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
async fn get_by_id(connection: Db, id: i32) -> Result<Json<Book>, String> {
    match connection
        .run(move |c| books::table.filter(books::id.eq(&id)).get_result(c))
        .await
        .map(Json)
    {
        Ok(book) => Ok(book),
        Err(_) => Err(format!("Only Dust Was To Found At The {}th Spot", id)),
    }
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

#[allow(dead_code)]
fn make_client() -> Client {
    Client::tracked(rocket()).expect("Valid Rocket Instance")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::fairing())
        .attach(AdHoc::config::<Config>())
        .register("/", catchers![not_found])
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
    use super::rocket;
    use crate::{make_client, models::Book};
    use rocket::{http::Status, serde::json::Json};

    #[test]
    fn test_index_status() {
        let client = make_client();

        let res = client.get(uri!(super::index)).dispatch();

        assert_eq!(Status::Ok, res.status());
    }

    #[test]
    fn test_index_content() {
        let client = make_client();

        let res = client.get(uri!(super::index)).dispatch();

        assert_eq!("Welcome To The Dusty Shelf", res.into_string().unwrap());
    }

    #[test]
    fn test_config_status() {
        let client = make_client();

        let res = client.get(uri!(super::get_config)).dispatch();

        assert_eq!(Status::Ok, res.status());
    }

    #[test]
    fn test_config_content() {
        let client = make_client();

        let res = client.get(uri!(super::get_config)).dispatch();

        assert_eq!(
            String::from("Hello etch1000, welcome to the club 24!"),
            res.into_string().unwrap()
        );
    }

    #[test]
    fn test_random_book_status() {
        let client = make_client();

        let res = client.get(uri!(super::get_random_book)).dispatch();

        assert_eq!(Status::Ok, res.status());
    }

    #[test]
    fn test_random_book_content() {
        let client = make_client();

        let exp_res = Json(Book {
            id: 0,
            title: String::from("Your Personal Diary"),
            author: String::from("You"),
            description: String::from("You know what this is about! We don't want to know! :)"),
            published: true,
        });

        let res = client.get(uri!(super::get_random_book)).dispatch();

        assert_eq!(exp_res.into_inner(), res.into_json().unwrap());
    }

    #[test]
    fn test_get_by_id_status() {
        let client = make_client();

        let res = client.get("/book/0").dispatch();

        assert_eq!(Status::Ok, res.status());
    }

    #[test]
    fn test_get_by_id_content() {
        let client = make_client();

        let exp_res = Json(
            Book {
                id: 0,
                title: String::from("Personal Diary"),
                author: String::from("You"),
                description: String::from("It's a secret! Shhhh..."),
                published: true,
            }
        );

        let res = client.get("/book/0").dispatch();

        assert_eq!(exp_res.into_inner(), res.into_json().unwrap());
    }

    #[test]
    fn test_get_by_id_404_status() {
        let client = make_client();

        let res = client.get("/book/0.0").dispatch();

        assert_eq!(Status::NotFound, res.status());
    }

    #[test]
    fn test_get_all_books_status() {
        let client = make_client();

        let res = client.get(uri!(super::get_all_books)).dispatch();

        assert_eq!(Status::Ok, res.status());
    }

    #[test]
    fn test_get_all_books_content() {
        let client = make_client();

        let exp_res = Json(vec![
          Book {
            id: 0,
            title: String::from("Personal Diary"),
            author: String::from("You"),
            description: String::from("It's a secret! Shhhh..."),
            published: true
          },
          Book {
            id: 1,
            title: String::from("Frankenstein"),
            author: String::from("Mary Shelley"),
            description: String::from("Frankenstein is an old classic about a scientist who creates a monster and the awful events he unintentionally causes"),
            published: true
          },
          Book {
            id: 2,
            title: String::from("To kill a mockingbird"),
            author: String::from("Harper Lee"),
            description: String::from("Scout, Arthur, Lawyer"),
            published: true
          },
          Book {
            id: 3,
            title: String::from("Nineteen Eighty-Four"),
            author: String::from("George Orwell"),
            description: String::from("The story takes place in an imagined future in the year 1984, when much of the world is in perpetual war. Great Britain, now known as Airstrip One, has become a province of the totalitarian superstate Oceania, which is led by Big Brother, a dictatorial leader supported by an intense cult of personality manufactured by the Party's Thought Police. Through the Ministry of Truth, the Party engages in omnipresent government surveillance, historical negationism, and constant propaganda to persecute individuality and independent thinking"),
            published: true
          }
        ]);

        let res = client.get(uri!(super::get_all_books)).dispatch();

        assert_eq!(exp_res.into_inner(), res.into_json::<Vec<Book>>().unwrap());
    }
}
