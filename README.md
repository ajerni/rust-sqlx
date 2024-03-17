# PostgreSQL Database Demo with SQLX in Rust

This project demonstrates a simple PostgreSQL database 💾 application written in Rust 🦀 using the SQLX library and serving it with Actix.

CRUD operations for your daily needs are covered 😊

Served live at: https://rust-sqlx.onrender.com/book/111

## Usage (HTML Requests for example with Postman)

    Get all: https://rust-sqlx.onrender.com
    Get one: https://rust-sqlx.onrender.com/book/111

    Create: POST https://rust-sqlx.onrender.com/create
    Add raw json to the body of the request i.e.:
    {"isbn":"999","title":"Rust Rocks Vol. 2","author":"Json Checker","metadata":{"avg_review":10.0,"tags":["cool", "stuff"]}}

    Update: PATCH https://rust-sqlx.onrender.com/update/999

    Delete: DELETE https://rust-sqlx.onrender.com/delete/999

Make sure to have a `.env` file on the top level of your projects path containing the following values:
`DATABASE_URL='postgres://user:password@host.com/db_name'`

It's based on the tutorial available here:
https://www.youtube.com/watch?v=TCERYbgvbq0 by Dream of Code.
