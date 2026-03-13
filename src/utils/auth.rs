use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use once_cell::sync::Lazy;

pub static JWT_SECRET: Lazy<String> = Lazy::new(|| std::env::var("JWT_SECRET").unwrap_or_else(|_| {
    "KageGmT2N9Hs685rEbfqVax7P3s4qjARAplB3PF5g07".to_string()
}));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

pub fn create_token(user_id: i32, username: String, role: String) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(24 * 60 * 60))
        .expect("valid time")
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("valid duration")
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id,
        username,
        role,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
