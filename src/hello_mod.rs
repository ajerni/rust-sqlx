use actix_web::{get, HttpResponse, Responder};

#[get("/hello")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world! Comming from separate file (see mod ... in main.rs)")
}
