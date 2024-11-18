use actix_web::{web, HttpResponse, Responder};
use crate::DbPool;
use crate::models::User;

pub async fn create_user(
    pool: web::Data<DbPool>,
    user: web::Json<User>,
) -> impl Responder {
    let pool = pool.lock().await;
    let result = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email) VALUES (?, ?) RETURNING id, name, email",
    )
        .bind(&user.name)
        .bind(&user.email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to insert user");

    web::Json(result)
}

pub async fn list_users(pool: web::Data<DbPool>) -> impl Responder {
    let pool = pool.lock().await;
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(&*pool)
        .await
        .expect("Failed to fetch users");

    web::Json(users)
}

pub async fn get_user(
    pool: web::Data<DbPool>,
    id: web::Path<i64>,
) -> impl Responder {
    let pool = pool.lock().await;
    let user = sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = ?")
        .bind(*id)
        .fetch_optional(&*pool)
        .await
        .expect("Failed to fetch user");

    match user {
        Some(u) => HttpResponse::Ok().json(u),
        None => HttpResponse::NotFound().finish(),
    }
}

pub async fn update_user(
    pool: web::Data<DbPool>,
    id: web::Path<i64>,
    user: web::Json<User>,
) -> impl Responder {
    let pool = pool.lock().await;
    let result = sqlx::query_as::<_, User>(
        "UPDATE users SET name = ?, email = ? WHERE id = ? RETURNING id, name, email",
    )
        .bind(&user.name)
        .bind(&user.email)
        .bind(*id)
        .fetch_optional(&*pool)
        .await
        .expect("Failed to update user");

    match result {
        Some(u) => HttpResponse::Ok().json(u),
        None => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_user(
    pool: web::Data<DbPool>,
    id: web::Path<i64>,
) -> impl Responder {
    let pool = pool.lock().await;
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(*id)
        .execute(&*pool)
        .await
        .expect("Failed to delete user");

    HttpResponse::Ok().json(serde_json::json!({ "status": "deleted" }))
}
