use actix_web::{delete, get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::db::SimpleDB;

/// Payload structure for the /set endpoint.
#[derive(Deserialize)]
pub struct SetRequest {
    pub key: String,
    pub value: String,
}

/// POST /set: Stores a key-value pair.
#[post("/set")]
async fn set_item(db: web::Data<Mutex<SimpleDB>>, item: web::Json<SetRequest>) -> impl Responder {
    let mut db = db.lock().unwrap();
    match db.set(item.key.clone(), item.value.clone()) {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"data": "Key set successfully"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

/// GET /get: Retrieves the value for a given key.
#[get("/get")]
async fn get_item(db: web::Data<Mutex<SimpleDB>>, info: web::Query<HashMap<String, String>>) -> impl Responder {
    if let Some(key) = info.get("key") {
        let mut db = db.lock().unwrap();
        match db.get(key) {
            Ok(value) => HttpResponse::Ok().json(serde_json::json!({"key": key, "value": value})),
            Err(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "Key not found"})),
        }
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({"error": "Missing 'key' parameter"}))
    }
}

/// DELETE /delete: Deletes a key-value pair.
#[delete("/delete")]
async fn delete_item(db: web::Data<Mutex<SimpleDB>>, info: web::Query<HashMap<String, String>>) -> impl Responder {
    if let Some(key) = info.get("key") {
        let mut db = db.lock().unwrap();
        match db.delete(key) {
            Ok(()) => HttpResponse::Ok().json(serde_json::json!({"data": "Key deleted successfully"})),
            Err(_) => HttpResponse::NotFound().json(serde_json::json!({"error": "Key not found"})),
        }
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({"error": "Missing 'key' parameter"}))
    }
}

/// GET /hello: A simple test route.
#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from server")
}

/// Initializes and registers all routes for the application.
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(hello);
    cfg.service(set_item);
    cfg.service(get_item);
    cfg.service(delete_item);
}
