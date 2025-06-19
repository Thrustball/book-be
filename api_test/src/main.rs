use std::str::FromStr;

use rocket::http::Status;
use rocket::response::status;
use rocket::serde::Serialize;
use rocket::tokio::time::{sleep, Duration};
use rocket::serde::{Deserialize, json::Json};
use rocket::{get, post, routes};
use rocket::http::Method; // 1.

use rocket::State;
use sqlx::prelude::FromRow;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Pool, Sqlite, SqlitePool};

use rocket_cors::{
    AllowedHeaders, AllowedOrigins, Cors, CorsOptions // 3.
};

#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
    sleep(Duration::from_secs(seconds)).await;
    format!("Waited for {} seconds", seconds)
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Serialize, Deserialize, FromRow)]
#[serde(crate = "rocket::serde")]
struct Book {
    title: String,
    author: String,
}

#[post("/book", data = "<book>")]
async fn put_book<'a>(book: Json<Book>, pool: &State<Pool<Sqlite>>) -> Status {
    println!("<{}> Book from <{}>", book.title, book.author);

    let insert_statement = "
    INSERT INTO books (title, author) VALUES ($1, $2);
    ";

    sqlx::query(&insert_statement)
        .bind(&book.title)
        .bind(&book.author)
        .execute(&**pool).await.unwrap();

    Status::Ok
}

#[get("/book")]
async fn get_book(pool: &State<Pool<Sqlite>>) -> Result<Json<Vec<Book>>, status::Custom<String>> {
    let keys= sqlx::query_as::<_, Book>("SELECT * FROM books")
        .fetch_all(&**pool)
        .await;

    match keys {
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
	id	INTEGER NOT NULL,
	title	TEXT NOT NULL,
	author	TEXT NOT NULL,
	created_on	DATETIME DEFAULT (datetime('now', 'localtime')),
	PRIMARY KEY(id AUTOINCREMENT)
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