use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::services::user_service::UserService;

#[derive(Deserialize)]
pub struct SignupPayload {
    email: String,
    password: String
}

#[derive(Serialize)]
pub struct SignupResponsePayload {
    message: String
}

pub async fn handler(
    State(service): State<UserService>,
    Json(payload): Json<SignupPayload>
) -> impl IntoResponse {
    match service.signup_execute(payload.email, payload.password).await {
        Ok(()) => {
            (StatusCode::CREATED).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(SignupResponsePayload{message: e.to_string()})).into_response()
        }
    }
}