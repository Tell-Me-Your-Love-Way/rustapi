use redis::{aio::ConnectionManager, AsyncCommands};
use sqlx::PgPool;
use tracing::info;

use crate::models::{errors::AppError, user_entity::User, user_repository::UserRepository};

#[derive(Clone)]
pub struct SqlxUserRepository {
    pool: PgPool,
    cache: ConnectionManager
}
const CACHED_PASSWORD_KEY_PREFIX: &str = "password:";
impl SqlxUserRepository {
    pub fn new(pool: PgPool, cache: ConnectionManager) -> Self{
        Self {pool, cache}
    }
    async fn create(&self, user: User) ->  Result<(), AppError> {
        sqlx::query!("INSERT INTO users (id, email, password) VALUES ($1, $2, $3)",user.get_id_as_uuid()?, user.get_email(), user.get_password())
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| {
                AppError::DatabaseError(e.to_string())
            })
    }
    async fn get_password(&self, email: String) ->  Result<String, AppError>{
        let mut cache = self.cache.clone();
        let result: Option<String> = cache
            .get(CACHED_PASSWORD_KEY_PREFIX.to_string()+&email)
            .await
            .map_err(|e| {
                info!("{}",e.to_string());
                AppError::CacheError(e.to_string())
            })?;
        match result {
            Some(cached_password) => Ok(cached_password),
            None => {    
                let password = sqlx::query_scalar!("SELECT password FROM users WHERE email = $1", email)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| {
                        AppError::DatabaseError(e.to_string())
                    })?;
                redis::cmd("SET")
                    .arg(&[CACHED_PASSWORD_KEY_PREFIX.to_string()+&email, password.clone()])
                    .exec_async(&mut cache)
                    .await
                    .map_err(|e| AppError::CacheError(e.to_string()))?;
                Ok(password)
            }
        }
        
    }
    async fn get_email(&self, id: String) ->  Result<String, AppError> {
        let id = uuid::Uuid::parse_str(&id).map_err(|e| AppError::ProviderError(e.to_string()))?;
        let email = sqlx::query_scalar!("SELECT email FROM users WHERE id = $1", id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(e.to_string())
            })?;
        Ok(email)
    }
    async fn change_password(&self, id: String, password: String) ->  Result<(), AppError> {
        let id = uuid::Uuid::parse_str(&id).map_err(|e| AppError::ProviderError(e.to_string()))?;
        sqlx::query!("UPDATE users SET password = $1 WHERE id = $2", password, id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| {
                AppError::DatabaseError(e.to_string())
            })
    }
    
}

#[async_trait::async_trait]
impl UserRepository for SqlxUserRepository {
    async fn create(&self, user: User) ->  Result<(), AppError>{
        self.create(user).await
    }
    async fn get_password(&self, email: String) ->  Result<String, AppError>{
        self.get_password(email).await
    }
    async fn get_email(&self, id: String) ->  Result<String, AppError>{
        self.get_email(id).await
    }
    async fn change_password(&self, id: String, password: String) ->  Result<(),AppError>{
        self.change_password(id, password).await
    }
}