use actix_web::{get, web, HttpResponse, Responder, web::Redirect};
use sqlx::PgPool;
use sqlx::postgres::PgListener;

#[get("/listen")]
pub async fn listen_to_pgsql(pool: web::Data<PgPool>) -> impl Responder {

    // Start the listener
    if let Err(e) = start_listener(&pool).await {
        eprintln!("Failed to start listener: {}", e);
    }

    HttpResponse::Ok().finish()
}

async fn start_listener(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = PgListener::connect_with(pool).await?; // Connect the PgListener
    listener.listen("mychannel").await?; // Listen to the specific channel

    // Loop to listen for notifications
    loop {
        let event = listener.recv().await?;
        handle_notification(event).await;
    }
}

// Function to handle incoming notifications
async fn handle_notification(event: sqlx::postgres::PgNotification) {
    // Perform actions based on the received notification
    println!("Notification successfully received: {:?}", event);
    println!("Got this message: {}", event.payload());
    if event.payload() == "plpgsql-trigger" {
        println!("plpsql is soooo coool!");
    }

    // Redirect within the main thread
    Redirect::to("/other").see_other();
}
