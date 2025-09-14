use crate::models::errors::AppError;


#[derive(Clone)]
pub struct User {
    id: String,
    email: String,
    password: String
}

impl User {
    pub fn new(incomming_id: impl Into<String>, incomming_email: impl Into<String>, incomming_password: impl Into<String>) -> Self {
        User { id: incomming_id.into(), email: incomming_email.into(), password: incomming_password.into() }
    }
    pub fn get_id(&self) -> &str {
        &self.id
    }
    pub fn get_id_as_uuid(&self) -> Result<uuid::Uuid, AppError> {
        uuid::Uuid::parse_str(&self.id).map_err(|_| AppError::ProviderError("uuid".to_owned()))
    }
    pub fn get_password(&self) -> &str {
        &self.password
    }
    pub fn get_email(&self) -> &str {
        &self.email
    }
}