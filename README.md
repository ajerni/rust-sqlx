# PostgreSQL Database Demo with SQLX in Rust

This project demonstrates a simple PostgreSQL database 💾 application written in Rust 🦀 using the SQLX library and serving it with Actix. The Actix webserver also provides endpoints connecting to ChatGPT using an llm-chain module and several other demo features.

CRUD operations for your daily needs are covered 😊

Served live at: https://rust-sqlx.onrender.com

## Usage (HTML Requests for example with Postman)

    Get all: https://rust-sqlx.onrender.com/books
    Get one: https://rust-sqlx.onrender.com/book/111

    Create: POST https://rust-sqlx.onrender.com/create
    Add raw json to the body of the request i.e.:
    {"isbn":"999","title":"Rust Rocks Vol. 2","author":"Json Checker","metadata":{"avg_review":10.0,"tags":["cool", "stuff"]}}

    Update: PATCH https://rust-sqlx.onrender.com/update/999
    Add raw json to the body of the request i.e.:
    {"isbn":"999","title":"Rust Rocks Vol. 3","author":"Json Checker","metadata":{"avg_review":9.0,"tags":["still", "cool"]}}

    Delete: DELETE https://rust-sqlx.onrender.com/delete/999

Make sure to have a `.env` file on the top level of your projects path containing the following values:
`DATABASE_URL='postgres://user:password@host.com/db_name'`

It's based on the tutorial available here:
https://www.youtube.com/watch?v=TCERYbgvbq0 by Dream of Code.

Startet as database example, this repo grows into my general purpose Rust/Actix/Postgres/llm anything demo. Basically serving as my main "Backend template".

I have also added a htmx frontend example under:
https://rust-sqlx.onrender.com/htmx and https://htmx.andierni.ch

Latest feature is the scoreboard - all logic handeld in PostgreSQL using views, triggers and functions in the database itself - see psql_helper.txt
