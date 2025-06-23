use rocket::futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Row, Result};

#[derive(Serialize, Deserialize, FromRow)]
pub struct Book {
    pub id: Option<i64>, // None before insertion, Some(id) after DB insert
    pub title: String,
    pub author: String,
    pub publishingdate: Option<String>,
    pub purchaseddate: Option<String>,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub price_new: Option<String>,
    pub price_bought: Option<String>,
    pub newused: Option<String>,
    pub pages: Option<String>,
    pub genres: sqlx::types::Json<Vec<String>>,
}

impl Book {
    pub async fn insert(&self, pool: &SqlitePool) -> Result<i64> {
        let row = sqlx::query(
            r#"
            INSERT INTO books (
                title, author, publishingdate, purchaseddate, publisher, isbn,
                price_new, price_bought, newused, pages, genres
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#
        )
        .bind(&self.title)
        .bind(&self.author)
        .bind(&self.publishingdate)
        .bind(&self.purchaseddate)
        .bind(&self.publisher)
        .bind(&self.isbn)
        .bind(&self.price_new)
        .bind(&self.price_bought)
        .bind(&self.newused)
        .bind(&self.pages)
        .bind(&self.genres)
        .fetch_one(pool)
        .await?;

        let id: i64 = row.try_get("id")?;

        Ok(id)
    }
}

impl Book {
    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Book>> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, title, author, publishingdate, purchaseddate, publisher, isbn,
                price_new, price_bought, newused, pages, genres
            FROM books
            WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            let book = Book {
                id: row.try_get("id")?,
                title: row.try_get("title")?,
                author: row.try_get("author")?,
                publishingdate: row.try_get("publishingdate")?,
                purchaseddate: row.try_get("purchaseddate")?,
                publisher: row.try_get("publisher")?,
                isbn: row.try_get("isbn")?,
                price_new: row.try_get("price_new")?,
                price_bought: row.try_get("price_bought")?,
                newused: row.try_get("newused")?,
                pages: row.try_get("pages")?,
                genres: row.try_get("genres")?,
            };
            Ok(Some(book))
        } else {
            Ok(None)
        }
    }
}

impl Book {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Book>> {
        let mut books = Vec::new();

        let mut rows = sqlx::query(
            r#"
            SELECT 
                id, title, author, publishingdate, purchaseddate, publisher, isbn,
                price_new, price_bought, newused, pages, genres
            FROM books
            "#
        )
        .fetch(pool);

        while let Some(row) = rows.try_next().await? {
            let book = Book {
                id: row.try_get("id")?,
                title: row.try_get("title")?,
                author: row.try_get("author")?,
                publishingdate: row.try_get("publishingdate")?,
                purchaseddate: row.try_get("purchaseddate")?,
                publisher: row.try_get("publisher")?,
                isbn: row.try_get("isbn")?,
                price_new: row.try_get("price_new")?,
                price_bought: row.try_get("price_bought")?,
                newused: row.try_get("newused")?,
                pages: row.try_get("pages")?,
                genres: row.try_get("genres")?,
            };
            books.push(book);
        }

        Ok(books)
    }
}
