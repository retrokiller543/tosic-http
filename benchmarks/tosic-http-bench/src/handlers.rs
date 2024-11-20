use crate::models::User;
use sqlx::{Pool, Sqlite};
use tosic_http::prelude::{BoxBody, Data, HttpResponse, Json, Path, Responder};

//#[post("/users")]
pub async fn create_user(
    pool: Data<Pool<Sqlite>>,
    user: Json<User>,
) -> impl Responder<Body = BoxBody> {
    let pool = pool.into_inner();
    let result = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email) VALUES (?, ?) RETURNING id, name, email",
    )
    .bind(&user.name)
    .bind(&user.email)
    .fetch_one(&*pool)
    .await
    .expect("Failed to insert user");

    HttpResponse::Ok().json(&result)
}

//#[get("/users")]
pub async fn list_users(pool: Data<Pool<Sqlite>>) -> impl Responder<Body = BoxBody> {
    let pool = pool.into_inner();
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(&*pool)
        .await
        .expect("Failed to fetch users");

    HttpResponse::Ok().json(&users)
}

//#[get("/users/{id}")]
pub async fn get_user(
    pool: Data<Pool<Sqlite>>,
    id: Path<(i64,)>,
) -> impl Responder<Body = BoxBody> {
    let pool = pool.into_inner();
    let id = id.into_inner().0;
    let user = sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .expect("Failed to fetch user");

    match user {
        Some(u) => HttpResponse::Ok().json(&u),
        None => HttpResponse::new(404),
    }
}

//#[put("/users/{id}")]
pub async fn update_user(
    pool: Data<Pool<Sqlite>>,
    id: Path<(i64,)>,
    user: Json<User>,
) -> impl Responder<Body = BoxBody> {
    let pool = pool.into_inner();
    let result = sqlx::query_as::<_, User>(
        "UPDATE users SET name = ?, email = ? WHERE id = ? RETURNING id, name, email",
    )
    .bind(&user.name)
    .bind(&user.email)
    .bind(id.into_inner().0)
    .fetch_optional(&*pool)
    .await
    .expect("Failed to update user");

    match result {
        Some(u) => HttpResponse::Ok().json(&u),
        None => HttpResponse::new(404),
    }
}

//#[delete("/users/{id}")]
pub async fn delete_user(
    pool: Data<Pool<Sqlite>>,
    id: Path<(i64,)>,
) -> impl Responder<Body = BoxBody> {
    let pool = pool.into_inner();
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id.into_inner().0)
        .execute(&*pool)
        .await
        .expect("Failed to delete user");

    HttpResponse::Ok().json(&serde_json::json!({ "status": "deleted" }))
}
