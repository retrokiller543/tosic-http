use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use crate::DbPool;
use crate::models::User;

pub async fn create_user(
    State(pool): State<DbPool>,
    Json(user): Json<User>,
) -> Json<User> {
    let pool = pool.lock().await;
    let result = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email) VALUES (?, ?) RETURNING id, name, email",
    )
        .bind(&user.name)
        .bind(&user.email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to insert user");

    Json(result)
}

pub async fn list_users(State(pool): State<DbPool>) -> Json<Vec<User>> {
    let pool = pool.lock().await;
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(&*pool)
        .await
        .expect("Failed to fetch users");

    Json(users)
}

pub async fn get_user(
    State(pool): State<DbPool>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<User>, StatusCode> {
    let pool = pool.lock().await;
    let user = sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .expect("Failed to fetch user");

    match user {
        Some(u) => Ok(Json(u)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_user(
    State(pool): State<DbPool>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(user): Json<User>,
) -> Result<Json<User>, StatusCode> {
    let pool = pool.lock().await;
    let result = sqlx::query_as::<_, User>(
        "UPDATE users SET name = ?, email = ? WHERE id = ? RETURNING id, name, email",
    )
        .bind(&user.name)
        .bind(&user.email)
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .expect("Failed to update user");

    match result {
        Some(u) => Ok(Json(u)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_user(
    State(pool): State<DbPool>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> axum::response::Json<serde_json::Value> {
    let pool = pool.lock().await;
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .expect("Failed to delete user");

    axum::response::Json(serde_json::json!({ "status": "deleted" }))
}
