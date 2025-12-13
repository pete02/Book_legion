use argon2::{
    Argon2, password_hash::{
        PasswordHash,
        PasswordVerifier,
        rand_core::{OsRng, RngCore}
    }
};
use std::fs;
use std::io;

use crate::models::{Claims, LoginRecord, UserRecord};


pub fn verify_login(login:&LoginRecord) -> io::Result<bool> {
    let data = fs::read_to_string("config/user.json")?;
    let user: UserRecord = serde_json::from_str(&data)?;

    if user.username != login.username {
        let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$\
                          AAAAAAAAAAAAAAAAAAAAAA$\
                          AAAAAAAAAAAAAAAAAAAA";
        let parsed = PasswordHash::new(dummy_hash).unwrap();
        let _ = Argon2::default().verify_password(b"dummy", &parsed);
        return Ok(false);
    }

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .expect("Stored hash is invalid");
    match Argon2::default().verify_password(login.password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}



use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};

pub fn generate_jwt(username: &str, secret: &[u8]) -> String {
    let now = Utc::now();
    let claims = Claims {
        sub: username.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::minutes(15)).timestamp() as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
        .expect("JWT generation failed")
}


pub fn generate_secret() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}


use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, errors::Result as JwtResult};
pub fn verify_jwt(token: &str, secret: &[u8]) -> JwtResult<String> {
    // Define validation rules
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    // Decode and validate the token
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)?;

    let now = Utc::now().timestamp() as usize;
    if token_data.claims.exp < now {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::ExpiredSignature,
        ));
    }

    Ok(token_data.claims.sub)
}