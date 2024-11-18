use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    id: Option<i64>,
    pub(crate) name: String,
    pub(crate) email: String,
}
