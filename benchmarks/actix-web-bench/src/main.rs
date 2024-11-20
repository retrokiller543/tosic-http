mod handlers;
mod models;

use crate::handlers::{create_user, delete_user, get_user, list_users, update_user};
use actix_web::{web, App, HttpServer, Responder};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tokio::sync::Mutex;

type DbPool = Arc<Mutex<sqlx::SqlitePool>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the in-memory SQLite database
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to the database");

    // Create the users table
    sqlx::query(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL
        );",
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");

    let shared_pool = Arc::new(Mutex::new(pool));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_pool.clone()))
            .service(
                web::resource("/users")
                    .route(web::post().to(create_user))
                    .route(web::get().to(list_users)),
            )
            .service(
                web::resource("/users/{id}")
                    .route(web::get().to(get_user))
                    .route(web::put().to(update_user))
                    .route(web::delete().to(delete_user)),
            )
    })
    .bind("127.0.0.1:3001")?
    .run()
    .await
}
