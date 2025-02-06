mod db;
mod routes;

use actix_web::{web, App, HttpServer};
use std::env;
use std::sync::Mutex;
use crate::db::SimpleDB;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the database.
    let db_path = "mydb.data";
    let simple_db = SimpleDB::open(db_path).expect("Failed to open database");

    // Wrap the database in a Mutex and share it via Actix Web's Data.
    let db_data = web::Data::new(Mutex::new(simple_db));

    // Read port from environment variable or default to 8080.
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("localhost:{}", port);

    println!("Starting server at http://{}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .configure(routes::init_routes) // Register our routes, including the test route
    })
    .bind(bind_address)?
    .run()
    .await
}
