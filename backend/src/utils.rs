use crate::config::Config;
use crate::errors::AppError;
use crate::models::auth::{Claims, User};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use sha2::{Digest, Sha256};
use argon2::{
    Argon2,
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};

// ─── Password hashing ─────────────────────────────────────────────────────────

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::InternalServerError)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|_| AppError::InternalServerError)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// ─── JWT ─────────────────────────────────────────────────────────────────────

pub fn create_access_token(user: &User, config: &Config) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = now + chrono::Duration::seconds(config.jwt_access_expiry_seconds);

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        name: user.name.clone(),
        role: user.role.clone(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::InternalServerError)
}

pub fn verify_access_token(token: &str, config: &Config) -> Result<Claims, AppError> {
    let key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    decode::<Claims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized("Invalid or expired access token".to_string()))
}

// ─── Refresh token (random hex, stored as SHA-256 hash) ──────────────────────

pub fn generate_refresh_token() -> String {
    let bytes: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen::<u8>()).collect();
    hex::encode(bytes) // 64-char hex string
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
