mod book_repository;

use std::str::FromStr;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::{json};
use rocket::tokio::time::{sleep, Duration};
use rocket::serde::{json::Json};
use rocket::{get, post, routes};
use rocket::http::Method; // 1.

use rocket::State;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Pool, Sqlite, SqlitePool};

use rocket_cors::{
    AllowedHeaders, AllowedOrigins, Cors, CorsOptions // 3.
};

use crate::book_repository::Book;

#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
    sleep(Duration::from_secs(seconds)).await;
    format!("Waited for {} seconds", seconds)
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/book", data = "<book>")]
async fn put_book<'a>(book: Json<Book>, pool: &State<Pool<Sqlite>>) -> Result<Json<Book>, status::Custom<String>> {
    let result = book.insert(pool).await;

    match result {
        Ok(id) => Ok(Json(Book{id: Some(id), ..book.into_inner()})),
        Err(e) => Err(status::Custom(Status::InternalServerError, format!("DB error: {}", e))),
    }
}

#[get("/book")]
async fn get_book(pool: &State<Pool<Sqlite>>) -> Result<Json<Vec<Book>>, status::Custom<String>> {
    let all_books = Book::get_all(pool).await;

    match all_books {
        Ok(r) => Ok(Json(r)),
        Err(e) => Err(status::Custom(Status::InternalServerError, format!("DB error: {}", e))),
    }
}



fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::All;

    CorsOptions { // 5.
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Put].into_iter().map(From::from).collect(), // 1.
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Access-Control-Allow-Origin",
            "Content-Type" // 6.
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

async fn init_db(pool: &Pool<Sqlite>) {
        let qry = 
    "CREATE TABLE IF NOT EXISTS books (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    publishingdate TEXT,
    purchaseddate TEXT,
    publisher TEXT,
    isbn TEXT,
    price_new TEXT,
    price_bought TEXT,
    newused TEXT,
    pages TEXT,
    genres TEXT -- Store JSON array as TEXT
    );";

    let res = sqlx::query(&qry).execute(pool).await;
    res.unwrap();
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let options = SqliteConnectOptions::from_str("sqlite://mydb.sqlite")
        .unwrap()
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options)
        .await
        .expect("Could not connect sql database");

    init_db(&pool).await;

    let _rocket = rocket::build()
        .manage(pool)
        .mount("/", routes![index, delay, put_book, get_book])
        .attach(make_cors())
        .launch()
        .await?;

    Ok(())
}