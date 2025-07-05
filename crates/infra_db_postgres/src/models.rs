use sqlx::{types::Uuid, FromRow};

#[derive(Debug, FromRow)]
pub(crate) struct UserRow {
    pub id: Uuid,
    pub name: String,
}
