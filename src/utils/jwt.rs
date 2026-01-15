use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user_id)
    pub username: String, // Username
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
}

/// Create JWT token
pub fn create_token(user_id: i32, username: &str, secret: &str) -> Result<String> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(24); // Token validity period 24 hours

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

/// Verify JWT token
pub fn validate_token(token: &str, secret: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
