use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, Row};
use std::env;
use std::error::Error;

#[derive(Debug, FromRow, Clone)]
struct Book {
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub metadata: Option<Json<Metadata>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Metadata {
    pub avg_review: f32,
    pub tags: Vec<String>,
}

async fn create(book: &Book, pool: &sqlx::PgPool) -> Result<&'static str, Box<dyn Error>> {
    let query = "INSERT INTO book (isbn, title, author, metadata) VALUES ($1, $2, $3, $4)";

    let mut book_to_insert = book.clone(); // Clone the book to avoid modifying the original
    if book_to_insert.metadata.is_none() {
        // Give it default values if metadata: None
        book_to_insert.metadata = Some(Json(Metadata {
            avg_review: 0.0,
            tags: vec![],
        }))
    }

    sqlx::query(query)
        .bind(&book_to_insert.isbn)
        .bind(&book_to_insert.title)
        .bind(&book_to_insert.author)
        .bind(Json(&book_to_insert.metadata))
        .execute(pool)
        .await?;

    Ok("Book created successfully.")
}

async fn get_book(isbn: &str, pool: &sqlx::PgPool) -> Result<Option<Book>, Box<dyn Error>> {
    let book = sqlx::query_as::<_, Book>(
        "SELECT book.isbn, book.title, book.author, book.metadata FROM book WHERE isbn = $1",
    )
    .bind(isbn)
    .fetch_optional(pool)
    .await?;

    Ok(book)
}

async fn get_all_books(pool: &sqlx::PgPool) -> Result<Vec<Book>, Box<dyn Error>> {
    let q = "SELECT isbn, title, author, metadata FROM book";
    let rows = sqlx::query(q).fetch_all(pool).await?;

    let books = rows
        .iter()
        .map(|row| Book {
            isbn: row.get("isbn"),
            title: row.get("title"),
            author: row.get("author"),
            metadata: row.get("metadata"),
        })
        .collect();

    Ok(books)
}

async fn get_books_by_author(author: &str, pool: &sqlx::PgPool) -> Result<Vec<Book>, Box<dyn Error>> {
    let q = "SELECT isbn, title, author, metadata FROM book WHERE author = $1";
    let rows = sqlx::query(q).bind(author).fetch_all(pool).await?;

    let books = rows
        .iter()
        .map(|row| Book {
            isbn: row.get("isbn"),
            title: row.get("title"),
            author: row.get("author"),
            metadata: row.get("metadata"),
        })
        .collect();

    Ok(books)
}

async fn update(book: &Book, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "UPDATE book SET title = $1, author = $2, metadata = $3 WHERE isbn = $4";

    sqlx::query(query)
        .bind(&book.title)
        .bind(&book.author)
        .bind(Json(&book.metadata))
        .bind(&book.isbn)
        .execute(pool)
        .await?;

    Ok(())
}

async fn delete(isbn: &str, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query_string = "DELETE FROM book WHERE isbn = $1";

    sqlx::query(query_string).bind(isbn).execute(pool).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = sqlx::postgres::PgPool::connect(&url.as_str()).await?;

    //sqlx::migrate!("./migrations").run(&pool).await?;

    // let book = Book {
    //     isbn: "9999".to_string(),
    //     title: "Rust Becoms Sucess".to_string(),
    //     author: "Andi Erni".to_string(),
    //     metadata: Some(Json(Metadata {
    //         avg_review: 3.5,
    //         tags: vec!["Programming".to_string(), "Rust".to_string()],
    //     })),
    // };

    // let successful_created = create(&book, &pool).await?;
    // println!("{}", successful_created);

    // let book: Option<Book> = get_book("999", &pool).await?;
    // match book {
    //     Some(b) => println!("Book: {:#?}", b),
    //     None => println!("Book not found."),
    // }

    let books: Vec<Book> = get_all_books(&pool).await?;
    println!("Books: {:?}", books);

    // let books: Vec<Book> = get_books_by_author("Andi Erni", &pool).await?;
    // println!("Books: {:#?}", books);

    // update(&book, &pool).await?;

    // delete("1111", &pool).await?;

    Ok(())
}
