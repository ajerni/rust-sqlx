use actix_web::{get, Responder, HttpResponse};

#[get("/dweet")]
pub async fn dweet() -> impl Responder {
    
    let thing_name = "ae_dweet_test";
    let url = format!("https://dweet.io/get/latest/dweet/for/{}", thing_name);
    let response = reqwest::Client::new()
        .get(url)
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap();

    let response_body = response.text().await.unwrap();

    HttpResponse::Ok().body(response_body)
}