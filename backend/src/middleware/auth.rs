use actix_web::{web, FromRequest, HttpRequest};
use std::future::{ready, Ready};

use crate::errors::AppError;
use crate::models::auth::Claims;
use crate::state::AppState;
use crate::utils::verify_access_token;

/// Extractor that validates the `Authorization: Bearer <token>` header.
/// Add `auth: AuthUser` to a handler signature to require authentication.
#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

impl AuthUser {
    pub fn claims(&self) -> &Claims {
        &self.0
    }

    pub fn user_id(&self) -> uuid::Uuid {
        self.0.sub.parse().expect("sub must be a valid UUID")
    }
}

impl FromRequest for AuthUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        // Extract Bearer token from Authorization header
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));

        let Some(token) = token else {
            return ready(Err(AppError::Unauthorized(
                "Missing Authorization header".to_string(),
            )));
        };

        // Get AppState from app data
        let Some(state) = req.app_data::<web::Data<AppState>>() else {
            return ready(Err(AppError::InternalServerError));
        };

        match verify_access_token(token, &state.config) {
            Ok(claims) => ready(Ok(AuthUser(claims))),
            Err(e) => ready(Err(e)),
        }
    }
}
