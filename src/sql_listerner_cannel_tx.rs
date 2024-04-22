use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use sqlx::postgres::PgListener;
use std::sync::mpsc::channel;

#[get("/listen")]
// Main function to start the listener and handle notifications
pub async fn listen_to_pgsql(pool: web::Data<PgPool>) -> impl Responder {
    // Create a channel to receive notifications
    let (tx, _rx) = channel();

    // Create a thread-safe wrapper for the channel transmitter
    let tx = Arc::new(Mutex::new(tx));

    // Clone the pool for async block
    let pool_clone = pool.clone();

    // Start the listener
    tokio::spawn(async move {
        if let Err(e) = start_listener(&pool_clone, tx.clone()).await {
            eprintln!("Failed to start listener: {}", e);
        }
    });

    HttpResponse::Ok().finish()
}

async fn start_listener(pool: &PgPool, tx: Arc<Mutex<std::sync::mpsc::Sender<()>>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = PgListener::connect_with(pool).await?; // Connect the PgListener
    listener.listen("mychannel").await?; // Listen to the specific channel

    // Create a new thread to listen for notifications
    loop {
        let event = listener.recv().await?;
        let tx_clone = tx.clone();

        // Spawn a new thread to handle each notification
        tokio::spawn(async move {
            // Send a signal through the channel to indicate a notification was received
            let _ = tx_clone.lock().unwrap().send(());

            // Handle the notification
            handle_notification(event).await;
        });
    }
}

// Function to handle incoming notifications
async fn handle_notification(event: sqlx::postgres::PgNotification) {
    // Perform actions based on the received notification
    println!("Notification successfully received: {:?}", event);
    println!("Got this message: {}", event.payload());
    // if event.payload() == "plpgsql-trigger"{
    //     println!("plpsql is soooo coool!");
    // }
}