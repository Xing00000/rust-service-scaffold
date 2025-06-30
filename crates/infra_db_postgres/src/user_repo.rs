use crate::error::DbError;
use crate::models::UserRow;
use application::ports::UserRepository; // RepoError removed
use async_trait::async_trait;
use domain::{error::DomainError, user::User}; // Added
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: Pool<Postgres>,
}

impl PostgresUserRepository {
    pub async fn new(url: &str, max_conn: u32) -> Result<Self, DbError> {
        let pool = PgPoolOptions::new()
            .max_connections(max_conn)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find(&self, id: &Uuid) -> Result<User, DomainError> {
        let row: UserRow = sqlx::query_as!(UserRow, "SELECT id, name FROM users WHERE id = $1", id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?; // Convert sqlx::Error -> DbError -> DomainError
        Ok(User {
            id: row.id,
            name: row.name,
        })
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query!(
            r#"INSERT INTO users (id, name) VALUES ($1, $2)
               ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name"#,
            user.id,
            user.name
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?; // Convert sqlx::Error -> DbError -> DomainError
        Ok(())
    }

    async fn shutdown(&self) {
        self.pool.close().await;
    }
}

fn map_sqlx_err(e: sqlx::Error) -> DomainError {
    use sqlx::Error::*;
    match &e {
        RowNotFound => DomainError::NotFound("user".into()),
        Database(db) if db.is_unique_violation() => {
            DomainError::Duplicate(db.message().to_string())
        }
        _ => DomainError::Unexpected(e.to_string()),
    }
}
