use sqlx::{types::Uuid, FromRow};

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub name: String,
}
