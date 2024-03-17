use actix_web::{delete, get, patch, post, web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, PgPool, Row};
use std::env;
use std::error::Error;

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
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

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[post("/create")]
async fn create(pool: web::Data<PgPool>, book: web::Json<Book>) -> impl Responder {
    match create_d(&book, &pool).await {
        Ok(message) => HttpResponse::Ok().body(message),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[get("/books")]
async fn get_all_books(pool: web::Data<PgPool>) -> impl Responder {
    let q = "SELECT isbn, title, author, metadata FROM book";
    let rows = match sqlx::query(q).fetch_all(pool.get_ref()).await {
        Ok(rows) => rows,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Database error: {}", e))
        }
    };

    let books = rows
        .iter()
        .map(|row| Book {
            isbn: row.get("isbn"),
            title: row.get("title"),
            author: row.get("author"),
            metadata: row.get("metadata"),
        })
        .collect::<Vec<Book>>();

    HttpResponse::Ok().json(books)
}

#[get("/book/{isbn}")]
async fn get_book_by_id(pool: web::Data<PgPool>, path: web::Path<(String,)>) -> impl Responder {
    let isbn = path.into_inner().0;
    match get_book_d(&isbn, &pool).await {
        Ok(Some(book)) => {
            let response_body = format!("Here is the book information retrieved by ISBN {}:\n\n{}", isbn, serde_json::to_string(&book).unwrap());
            HttpResponse::Ok().body(response_body)
          },
        //Ok(Some(book)) => HttpResponse::Ok().json(book),
        Ok(None) => HttpResponse::NotFound().body("Book not found."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[patch("/update/{isbn}")]
async fn update_book(
    pool: web::Data<PgPool>,
    path: web::Path<(String,)>,
    new_book: web::Json<Book>,
) -> impl Responder {
    let isbn = path.into_inner().0;
    match get_book_d(&isbn, &pool).await {
        Ok(Some(mut book)) => {
            book.title = new_book.title.clone();
            book.author = new_book.author.clone();
            book.metadata = new_book.metadata.clone();

            match update_d(&book, &pool).await {
                Ok(_) => HttpResponse::Ok().body("Book updated successfully."),
                Err(e) => {
                    HttpResponse::InternalServerError().body(format!("Error updating book: {}", e))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().body("Book not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[delete("/delete/{isbn}")]
async fn delete_book(pool: web::Data<PgPool>, path: web::Path<(String,)>) -> impl Responder {
    let isbn = path.into_inner().0;
    match delete_d(&isbn, &pool).await {
        Ok(_) => HttpResponse::Ok().body("Book deleted successfully."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error deleting book: {}", e)),
    }
}

// The following methods (_d for direct) are also for direct use with sqlx / all off them are
// separately implementet or called from the server above (Main diff: sqlx::Pool vs. web::Data<PgPool>)

async fn create_d(book: &Book, pool: &sqlx::PgPool) -> Result<&'static str, Box<dyn Error>> {
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

async fn get_book_d(isbn: &str, pool: &sqlx::PgPool) -> Result<Option<Book>, Box<dyn Error>> {
    let book = sqlx::query_as::<_, Book>(
        "SELECT book.isbn, book.title, book.author, book.metadata FROM book WHERE isbn = $1",
    )
    .bind(isbn)
    .fetch_optional(pool)
    .await?;

    Ok(book)
}

async fn get_all_books_d(pool: &sqlx::PgPool) -> Result<Vec<Book>, Box<dyn Error>> {
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

async fn get_books_by_author_d(
    author: &str,
    pool: &sqlx::PgPool,
) -> Result<Vec<Book>, Box<dyn Error>> {
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

async fn update_d(book: &Book, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
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

async fn delete_d(isbn: &str, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query_string = "DELETE FROM book WHERE isbn = $1";

    sqlx::query(query_string).bind(isbn).execute(pool).await?;

    Ok(())
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = sqlx::postgres::PgPool::connect(&url.as_str()).await?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(create)
            .service(get_all_books)
            .service(get_book_by_id)
            .service(update_book)
            .service(delete_book)
            .route("/hey", web::get().to(manual_hello))
            .service(fs::Files::new("/other", "./static").index_file("other.html"))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    //.bind(("127.0.0.1", 8080))?
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

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

    // let successful_created = create_d(&book, &pool).await?;
    // println!("{}", successful_created);

    // let book: Option<Book> = get_book_d("999", &pool).await?;
    // match book {
    //     Some(b) => println!("Book: {:#?}", b),
    //     None => println!("Book not found."),
    // }

    // let books: Vec<Book> = get_all_books_d(&pool).await?;
    // println!("Books: {:?}", books);

    // let books: Vec<Book> = get_books_by_author_d("Andi Erni", &pool).await?;
    // println!("Books: {:#?}", books);

    // update_d(&book, &pool).await?;

    // delete_d("1111", &pool).await?;

    Ok(())
}
