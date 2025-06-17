use std::str::FromStr;

use rocket::http::{ContentType, Status};
use rocket::tokio::time::{sleep, Duration};
use rocket::serde::{Deserialize, json::Json};
use rocket::{get, post, put, routes, Response};
use rocket::http::Method; // 1.

use rocket_cors::{
    AllowedHeaders, AllowedOrigins, Error, // 2.
    Cors, CorsOptions // 3.
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

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Book {
    title: String,
    author: String
}

#[post("/book", data = "<book>")]
fn put_book<'a>(book: Json<Book>) -> Status {
    println!("<{}> Book from <{}>", book.title, book.author);

    Status::Ok
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

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    let _rocket = rocket::build()
        .mount("/", routes![index, delay, put_book])
        .attach(make_cors())
        .launch()
        .await?;

    Ok(())
}