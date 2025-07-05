use crate::error::DbError;
use crate::models::UserRow;
use contracts::{DomainError, User, UserId};
use domain::UserRepository;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{future::Future, pin::Pin};
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

impl UserRepository for PostgresUserRepository {
    fn find(&self, id: &UserId) -> Pin<Box<dyn Future<Output = Result<User, DomainError>> + Send + '_>> {
        let id_string = id.as_str().to_string();
        let pool = self.pool.clone();
        Box::pin(async move {
            let uuid = Uuid::parse_str(&id_string)
                .map_err(|_| DomainError::InvalidOperation { message: "Invalid ID format".to_string() })?;
            
            let row: Result<UserRow, sqlx::Error> = sqlx::query_as(
                "SELECT id, name FROM users WHERE id = $1"
            )
            .bind(uuid)
            .fetch_one(&pool)
            .await;
            
            let row = row.map_err(|e| DomainError::from(DbError::from(e)))?;
            let user_id = UserId::from_string(row.id.to_string());
            User::new(user_id, row.name)
        })
    }

    fn save(&self, user: &User) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + '_>> {
        let id_string = user.id.as_str().to_string();
        let name = user.name.clone();
        let pool = self.pool.clone();
        Box::pin(async move {
            let uuid = Uuid::parse_str(&id_string)
                .map_err(|_| DomainError::InvalidOperation { message: "Invalid ID format".to_string() })?;
            
            sqlx::query(
                r#"INSERT INTO users (id, name) VALUES ($1, $2)
                   ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name"#
            )
            .bind(uuid)
            .bind(name)
            .execute(&pool)
            .await
            .map_err(|e| DomainError::from(DbError::from(e)))?;
            Ok(())
        })
    }

    fn shutdown(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            pool.close().await;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_conversion() {
        let uuid = Uuid::now_v7();
        let user_row = UserRow {
            id: uuid,
            name: "Test User".to_string(),
        };

        let user_id = UserId::from_string(uuid.to_string());
        let user = User::new(user_id.clone(), user_row.name.clone()).unwrap();

        assert_eq!(user.id, user_id);
        assert_eq!(user.name, user_row.name);
    }
}
