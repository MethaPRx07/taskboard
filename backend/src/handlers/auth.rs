use actix_web::{web, HttpResponse};
use chrono::Utc;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::auth::{
    AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, TokenResponse,
};
use crate::repository::auth as auth_repo;
use crate::state::AppState;
use crate::utils::{
    create_access_token, generate_refresh_token, hash_password, hash_token, verify_password,
};

// ─── POST /api/v1/auth/register ──────────────────────────────────────────────

pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;

    // Check email uniqueness
    if auth_repo::find_user_by_email(&state.db, &body.email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict(format!(
            "Email '{}' is already registered",
            body.email
        )));
    }

    let password_hash = hash_password(&body.password)?;
    let user = auth_repo::create_user(&state.db, &body.email, &body.name, &password_hash).await?;

    // Issue tokens on register
    let access_token = create_access_token(&user, &state.config)?;
    let refresh_token = generate_refresh_token();
    let token_hash = hash_token(&refresh_token);
    let expires_at = Utc::now()
        + chrono::Duration::seconds(state.config.jwt_refresh_expiry_seconds);

    auth_repo::create_refresh_token(&state.db, user.id, &token_hash, expires_at).await?;

    Ok(HttpResponse::Created().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_access_expiry_seconds,
        user: user.into(),
    }))
}

// ─── POST /api/v1/auth/login ─────────────────────────────────────────────────

pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()?;

    let user = auth_repo::find_user_by_email(&state.db, &body.email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    if !user.is_active {
        return Err(AppError::Forbidden("Account is deactivated".to_string()));
    }

    if !verify_password(&body.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("Invalid email or password".to_string()));
    }

    let access_token = create_access_token(&user, &state.config)?;
    let refresh_token = generate_refresh_token();
    let token_hash = hash_token(&refresh_token);
    let expires_at = Utc::now()
        + chrono::Duration::seconds(state.config.jwt_refresh_expiry_seconds);

    auth_repo::create_refresh_token(&state.db, user.id, &token_hash, expires_at).await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_access_expiry_seconds,
        user: user.into(),
    }))
}

// ─── POST /api/v1/auth/refresh ───────────────────────────────────────────────

pub async fn refresh(
    state: web::Data<AppState>,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, AppError> {
    let token_hash = hash_token(&body.refresh_token);

    let stored = auth_repo::find_refresh_token(&state.db, &token_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid or expired refresh token".to_string()))?;

    let user = auth_repo::find_user_by_id(&state.db, stored.user_id)
        .await?
        .ok_or(AppError::InternalServerError)?;

    if !user.is_active {
        return Err(AppError::Forbidden("Account is deactivated".to_string()));
    }

    let access_token = create_access_token(&user, &state.config)?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_access_expiry_seconds,
    }))
}

// ─── POST /api/v1/auth/logout ────────────────────────────────────────────────

pub async fn logout(
    state: web::Data<AppState>,
    _auth: AuthUser,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, AppError> {
    let token_hash = hash_token(&body.refresh_token);
    auth_repo::delete_refresh_token(&state.db, &token_hash).await?;

    Ok(HttpResponse::NoContent().finish())
}

// ─── GET /api/v1/auth/me ─────────────────────────────────────────────────────

pub async fn me(
    state: web::Data<AppState>,
    auth: AuthUser,
) -> Result<HttpResponse, AppError> {
    let user = auth_repo::find_user_by_id(&state.db, auth.user_id())
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(crate::models::auth::UserResponse::from(user)))
}
