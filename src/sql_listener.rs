use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use sqlx::postgres::PgListener;
use webbrowser::open;

fn _go_to_page() {
  let url = "https://www.andierni.ch";
  open(url).unwrap();
}

fn open_message_box() {
    let html = r#"
        <html>
            <body>
                <h1>Data entered in DB</h1>
                <p>This temporary file was created uppon notification from PostgreSQL.</p>
                <button onclick="window.close()">OK</button>
            </body>
        </html>
    "#;

    // creates a temporary file on the heap (so it is automatically deleted again when it goes out of scope)
    let dir = tempfile::TempDir::new().unwrap();
    let file_path = dir.path().join("message.html");
    std::fs::write(&file_path, html).unwrap();

    webbrowser::open(&format!("file://{}", file_path.display())).unwrap();

    // Keep the temporary file around for 1 second
    std::thread::sleep(std::time::Duration::from_secs(1));
}

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
        println!("plpgql is soooo coool!");
    }
    
    //TODO: use WebSockets or Server Sent Events to trigger reload of fetch() in htmx.html (so far polling with htmx every 2s is used)
    
    //go_to_page();
    open_message_box();
    
}
