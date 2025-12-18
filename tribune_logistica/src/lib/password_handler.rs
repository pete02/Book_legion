use argon2::{
    Argon2, password_hash::{
        PasswordHash,
        PasswordVerifier,
        rand_core::{OsRng, RngCore}
    }
};

use std::io;

use crate::models::{Claims, LoginRecord, UserRecord};


pub fn verify_login(login:&LoginRecord, data:String) -> io::Result<bool> {
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
use chrono::{TimeDelta, Utc};

pub fn generate_jwt(username: &str, secret: &[u8], delta:TimeDelta) -> String {
    let now = Utc::now();
    let claims = Claims {
        sub: username.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + delta).timestamp() as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
        .expect("JWT generation failed")
}
use base64::{engine::general_purpose, Engine as _};
pub fn generate_and_store_refresh_token(
    username: &str,
    password_data:String
) -> io::Result<(String, String)> {
    let mut bytes = vec![0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token = general_purpose::URL_SAFE_NO_PAD.encode(&bytes);

    let mut user: UserRecord = serde_json::from_str(&password_data)?;

    if user.username != username {
        return Err(io::Error::new(io::ErrorKind::NotFound, "User not found"));
    }
    user.refresh_token=token.clone();
    let json = serde_json::to_string_pretty(&user)?;

    Ok((token, json))
}

pub fn check_refesh_token(username: &str, token: &str, delta:TimeDelta, secret: &[u8], password_data:String)-> io::Result<(String, (String,String))>{
    let user: UserRecord = serde_json::from_str(&password_data)?;

    if user.username!=username{
        return Err(io::Error::new(io::ErrorKind::NotFound, "User not found"));
    }

    if user.refresh_token!=token{
        return Err(io::Error::new(io::ErrorKind::NotFound, "incorrect refesh token"));
    }

    let refresh=generate_and_store_refresh_token(username, password_data)?;
    let access=generate_jwt(username, secret, delta);

    Ok((access,refresh))
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

pub fn generate_secret() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}