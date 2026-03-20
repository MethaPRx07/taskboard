use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::auth::{RefreshToken, User};

// ─── User queries ─────────────────────────────────────────────────────────────

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, email, name, password_hash, role, is_active, created_at, updated_at
           FROM users
           WHERE email = $1"#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, email, name, password_hash, role, is_active, created_at, updated_at
           FROM users
           WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    name: &str,
    password_hash: &str,
) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (email, name, password_hash)
           VALUES ($1, $2, $3)
           RETURNING id, email, name, password_hash, role, is_active, created_at, updated_at"#,
    )
    .bind(email)
    .bind(name)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

// ─── Refresh token queries ────────────────────────────────────────────────────

pub async fn create_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: chrono::DateTime<Utc>,
) -> Result<RefreshToken, AppError> {
    let rt = sqlx::query_as::<_, RefreshToken>(
        r#"INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
           VALUES ($1, $2, $3)
           RETURNING id, user_id, token_hash, expires_at, created_at"#,
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(rt)
}

pub async fn find_refresh_token(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<RefreshToken>, AppError> {
    let rt = sqlx::query_as::<_, RefreshToken>(
        r#"SELECT id, user_id, token_hash, expires_at, created_at
           FROM refresh_tokens
           WHERE token_hash = $1 AND expires_at > NOW()"#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;

    Ok(rt)
}

pub async fn delete_refresh_token(pool: &PgPool, token_hash: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1")
        .bind(token_hash)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_all_user_refresh_tokens(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}
