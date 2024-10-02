use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const JWT_EXPIRATION_DURATION_IN_HOURS: i64 = 12;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String,
    sub: String,
    exp: i64,
}

pub fn create_jwt(secret: &[u8], email: String) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        aud: String::from("set it later"), //TODO: handle this later
        sub: email,
        exp: (Utc::now() + Duration::hours(JWT_EXPIRATION_DURATION_IN_HOURS)).timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )?;
    Ok(token)
}

pub fn get_email_from_token(
    secret: &[u8],
    token: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    //todo!("impl");
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )?;
    Ok(token_data.claims.aud)
}
