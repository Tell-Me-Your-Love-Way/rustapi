use std::sync::Arc;

use bcrypt::{hash, verify};
use josekit::{jwe::{ alg::ecdh_es::EcdhEsJweAlgorithm::EcdhEs, JweHeader}, jwt::{self, JwtPayload}};


use crate::{models::{errors::AppError, user_entity::User, user_repository::UserRepository}, repository::user_repository::SqlxUserRepository};

#[derive(Clone)]
pub struct UserService {
    repo: SqlxUserRepository,
    privkey: Arc<Vec<u8>>,
    pubkey: Arc<Vec<u8>>
}

impl UserService {
    pub fn new(repo: SqlxUserRepository, privkey: Arc<Vec<u8>>, pubkey: Arc<Vec<u8>>) -> Self {
        Self { repo, privkey, pubkey }
    }
    pub async fn signin_execute(&self, incomming_email: impl Into<String>, incomming_password: impl Into<String>) -> Result<String, AppError> {
        let email = incomming_email.into();
        let retrieved_password = self.repo.get_password(email.clone()).await?;
        let valid = verify(&incomming_password.into(), &retrieved_password).map_err(|e| AppError::ProviderError(e.to_string()))?;
        match valid {
            true => {
                let mut payload = JwtPayload::new();
                payload.set_subject(email);
                let mut header = JweHeader::new();
                header.set_issuer("Ryder");                
                header.set_token_type("JWT");
                header.set_content_encryption("A128CBC-HS256");
                let encrypter = EcdhEs.encrypter_from_pem(&*self.pubkey.clone()).map_err(|e| AppError::ProviderError(e.to_string()))?;
                let token = jwt::encode_with_encrypter(&payload, &header, &encrypter).map_err(|e| AppError::ProviderError(e.to_string()))?;
                Ok(token)
            },
            false => {
                Err(AppError::ValidationError("Wrong Credentials".to_string()))
            }
        }
        
    }
    pub async fn signup_execute(&self, incomming_email: impl Into<String>, incomming_password: impl Into<String>) -> Result<(), AppError> {
        let provided_id = uuid::Uuid::now_v7().to_string();
        let provided_password = hash(incomming_password.into(), 8).map_err(|e| AppError::ProviderError(e.to_string()))?;
        let user = User::new(provided_id, incomming_email.into(), provided_password);
        self.repo.create(user).await
    }
}