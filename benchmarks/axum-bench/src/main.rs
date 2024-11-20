mod handlers;
mod models;

use crate::handlers::*;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tokio::sync::Mutex;

type DbPool = Arc<Mutex<sqlx::SqlitePool>>;

#[tokio::main]
async fn main() {
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

    // Build the router
    let app = Router::new()
        .route("/users", post(create_user).get(list_users))
        .route(
            "/users/:id",
            get(get_user).put(update_user).delete(delete_user),
        )
        .with_state(shared_pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
