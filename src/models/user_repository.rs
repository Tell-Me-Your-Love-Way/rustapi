use crate::models::{errors::AppError, user_entity::User};
#[async_trait::async_trait]
pub trait UserRepository {
    async fn create(&self, user: User) ->  Result<(), AppError>;
    async fn get_password(&self, email: String) ->  Result<String, AppError>;
    async fn get_email(&self, id: String) ->  Result<String, AppError>;
    async fn change_password(&self, id: String, password: String) ->  Result<(),AppError>;
}