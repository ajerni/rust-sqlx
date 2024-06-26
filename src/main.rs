use actix_cors::Cors;
use actix_files as fs;
use actix_web::http::header;
use actix_web::{delete, get, patch, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, PgPool, Row};
use std::env;
use std::error::Error;

mod llm_chain;
use crate::llm_chain::run_llm_chain;

mod hello_mod;
use crate::hello_mod::hello;

mod scoreboard;
use crate::scoreboard::*;

mod sql_listener;
use crate::sql_listener::*;

mod dweet;
use crate::dweet::*;

mod teratemplates;
use crate::teratemplates::*;

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

/// FromData struct is used for extractors web::Form and web::Query (params)
#[derive(Debug, Deserialize)]
struct FormData {
    isbn: String,
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

#[get("/param")] //call as param: http://xxx/param?isbn=123 / FormData is a struct that matches the params pattern
async fn get_book_from_param(
    info: web::Query<FormData>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let isbn = info.isbn.as_str();
    match get_book_d(isbn, &pool).await {
        Ok(Some(book)) => {
            let response_body = format!(
                "Here is the book information retrieved by ISBN {}:\n\n{}",
                isbn,
                serde_json::to_string(&book).unwrap()
            );
            HttpResponse::Ok().body(response_body)
        }
        //Ok(Some(book)) => HttpResponse::Ok().json(book),
        Ok(None) => HttpResponse::NotFound().body("Book not found."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

async fn handle_form(form: web::Form<FormData>, pool: web::Data<PgPool>) -> impl Responder {
    let isbn = form.isbn.as_str();

    match get_book_d(isbn, &pool).await {
        Ok(Some(book)) => {
            let response_body = format!(
                "Here is the book information retrieved by ISBN {}:\n\n{}",
                isbn,
                serde_json::to_string(&book).unwrap()
            );
            HttpResponse::Ok().body(response_body)
        }
        //Ok(Some(book)) => HttpResponse::Ok().json(book),
        Ok(None) => HttpResponse::Ok().body("Book not found."),
        Err(e) => {
            dbg!(&isbn);
            //HttpResponse::Ok().body("Book not found.")
            HttpResponse::InternalServerError().body(format!("Error: {}", e))
        }
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
            let response_body = format!(
                "Here is the book information retrieved by ISBN {}:\n\n{}",
                isbn,
                serde_json::to_string(&book).unwrap()
            );
            HttpResponse::Ok().body(response_body)
        }
        //Ok(Some(book)) => HttpResponse::Ok().json(book),
        Ok(None) => HttpResponse::NotFound().body("Book not found."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[get("/bookjson/{isbn}")]
async fn get_book_by_id_json(
    pool: web::Data<PgPool>,
    path: web::Path<(String,)>,
) -> impl Responder {
    let isbn = path.into_inner().0;
    match get_book_d(&isbn, &pool).await {
        Ok(Some(book)) => HttpResponse::Ok().body(serde_json::to_string(&book).unwrap()),
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

// this is a separate new section to test htmx

#[get("/htmxtest")]
async fn htmxtest() -> impl Responder {
    let my_whatever = String::from("💾 SERVER-DATA here!"); // Replace with your data fetching logic

    let response_body = format!(
        "<h3>I am pure HTML returned from the server...</h3>
        <p>...demonstrating the use of <strong>htmx</strong> and <strong>alpinejs</strong> in the 🖥️ frontend...</p>
        <p>...accessing 🗄️ <strong>Rust Actix webserver</strong> at the backend: {}", my_whatever);

    HttpResponse::Ok().body(response_body)
}

// this is to manually update the highscore data of my Bevy game (bevy.andierni.ch)

#[derive(Debug, Deserialize)]
struct FormDataScoreUpdate {
    name: String,
    highscore: String,
}

#[patch("/updatehighscore")]
async fn update_highscore(
    pool: web::Data<PgPool>,
    form: web::Form<FormDataScoreUpdate>,
) -> impl Responder {
    let book_new_highscore = Book {
        isbn: "9901".to_string(), // 9901 is used in the Bevy Game to store the highscore
        title: "Bevy highscore".to_string(),
        author: "updated via htmx".to_string(),
        metadata: Some(Json(Metadata {
            avg_review: 6.0,
            tags: vec![form.name.to_string(), form.highscore.to_string()],
        })),
    };

    match update_d(&book_new_highscore, &pool).await {
        Ok(_) => HttpResponse::Ok().body("Book updated successfully."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error updating book: {}", e)),
    }
}

// other unrelated endpoint test to show that just anything can be done on the server:

#[get("/addfive/{number}")]
async fn addfive(path: web::Path<(String,)>) -> impl Responder {
    let number = path.into_inner().0.parse::<u8>(); // Parse the string into a u8
                                                    // Check if parsing was successful
    if let Ok(number) = number {
        let r = number + 5;
        HttpResponse::Ok().body(format!("Result: {}", r))
    } else {
        // Handle parsing error here, e.g. return BadRequest
        HttpResponse::BadRequest().body("Invalid number format")
    }
}

// LIVE TEXT CHANGE WITH HTMX EVENT IN HEADER

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
struct TextInput {
    livetext: String,
}

#[patch("/livetextchange")]
async fn livetextchange(form: web::Form<TextInput>, pool: web::Data<PgPool>) -> impl Responder {
    //save the new livetext from form to the DB:
    let new_text = form.livetext.clone();

    //Two step process to save: callinge separate sqlx function allows to check for result here and trigger event only after DB entry has sucessfully finsished
    match update_livetext(new_text, &pool).await {
        Ok(_) => HttpResponse::Ok()
            //send event via response header to trigger hx-get in the html code (hx-trigger="new-text-entered-event from:body")
            .append_header(("HX-Trigger", "new-text-entered-event"))
            .finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error updating book: {}", e)),
    }
}

async fn update_livetext(text: String, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "UPDATE livedata SET value = $1 WHERE key = 'livetext'";

    sqlx::query(query).bind(&text).execute(pool).await?;

    Ok(())
}

#[derive(Serialize, Deserialize, FromRow)]
struct LiveText {
    value: String,
}

#[get("/getlivetext")]
pub async fn getlivetext(pool: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    let row: LiveText = sqlx::query_as("SELECT value FROM livedata WHERE key='livetext'")
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(row.value))
}

// LLM-CHAIN / ChatGPT demo
#[derive(Debug, Deserialize)]
struct AIInput {
    city: String,
    country: String,
}

// call with query params like: http://localhost:8080/ai?city=Stockholm&country=Sweden
#[get("/ai")]
async fn ai_get(params: web::Query<AIInput>) -> HttpResponse {
    let city = params.city.to_string();
    let country = params.country.to_string();

    println!(
        "Received via params --> City: {} / Country: {}",
        city, country
    );

    let response = run_llm_chain(city, country).await;

    match response {
        Ok(result) => {
            let final_res = result[11..].to_string(); //removing the leading text 'Assistant:'
            HttpResponse::Ok().body(final_res)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error from AI: {}", e)),
    }
}

// call with 'params' taken from frontend form
#[post("/ai")]
async fn ai_post(params: web::Form<AIInput>) -> HttpResponse {
    let city = params.city.to_string();
    let country = params.country.to_string();

    println!(
        "Received via params --> City: {} / Country: {}",
        city, country
    );

    let response = run_llm_chain(city, country).await;

    match response {
        Ok(result) => {
            let final_res = result[11..].to_string(); //removing the leading text 'Assistant:'
            HttpResponse::Ok().body(final_res)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error from AI: {}", e)),
    }
}

// here is the Actix server itself:

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = sqlx::postgres::PgPool::connect(url.as_str()).await?;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_origin("https://rust-sqlx.onrender.com")
            .allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b".onrender.com"))
            .allowed_origin("https://bevy.andierni.ch")
            .allowed_origin("https://htmx.andierni.ch")
            .allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b".andierni.ch"))
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
            ])
            .allowed_header(header::CONTENT_TYPE)
            .allow_any_header()
            .allow_any_method()
            .expose_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(htmxtest)
            .service(update_highscore)
            .service(addfive)
            .service(set_scoreboard)
            .service(set_scoreboard_form)
            .service(get_scoreboard)
            .service(get_topscorer)
            .service(livetextchange)
            .service(getlivetext)
            .service(listen_to_pgsql)
            .service(dweet)
            .service(teratemplates)
            .service(edit)
            .service(ai_get)
            .service(ai_post)
            .service(create)
            .service(get_all_books)
            .service(get_book_by_id)
            .service(get_book_by_id_json)
            .route("/form-handler", web::post().to(handle_form))
            .service(get_book_from_param)
            .service(update_book)
            .service(delete_book)
            .route("/hey", web::get().to(manual_hello))
            .service(fs::Files::new("/nextproject", "./static").index_file("nextproject.html"))
            .service(fs::Files::new("/other", "./static").index_file("other.html"))
            .service(fs::Files::new("/htmx", "./static").index_file("htmx.html"))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    //.bind(("127.0.0.1", 8080))? // 0.0.0.0 needed on render.com, works also as localhost
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