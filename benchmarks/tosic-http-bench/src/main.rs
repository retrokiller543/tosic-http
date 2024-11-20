#![feature(impl_trait_in_assoc_type)]

use std::io;
use tosic_http::prelude::Method;
use tosic_http::server::builder::HttpServerBuilder;

mod handlers;
mod models;

use crate::handlers::*;

#[tokio::main]
async fn main() -> io::Result<()> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to the database");

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

    let server = HttpServerBuilder::default()
        .app_state(pool)
        .service_method(Method::POST, "/users", create_user)
        .service_method(Method::GET, "/users", list_users)
        .service_method(Method::GET, "/users/{id}", get_user)
        .service_method(Method::PUT, "/users/{id}", update_user)
        .service_method(Method::DELETE, "/users/{id}", delete_user)
        .bind("0.0.0.0:3002")
        .build()
        .await?;

    match server.serve().await {
        Ok(_) => (),
        Err(e) => panic!("Failed to serve: {}", e),
    }

    Ok(())
}
