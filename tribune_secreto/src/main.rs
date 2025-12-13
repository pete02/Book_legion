use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher,
        SaltString,
        PasswordHash,
        PasswordVerifier
    },
    Argon2,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

#[derive(Serialize, Deserialize)]
struct UserRecord {
    username: String,
    password_hash: String,
}

fn main(){
    let _=make_user("username", "password");
}


fn make_user(username: &str, password: &str)->io::Result<()>{
    // Generate a cryptographically secure random salt
    let salt = SaltString::generate(&mut OsRng);

    // Argon2id with default safe parameters
    let argon2 = Argon2::default();

    // Hash the password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Password hashing failed")
        .to_string();

    let user = UserRecord {
        username: username.to_string(),
        password_hash,
    };

    let json = serde_json::to_string_pretty(&user)
        .expect("JSON serialization failed");

    fs::write("user.json", json)?;

    println!("User record written to user.json");

    Ok(())
}


pub fn verify_login(username: &str, password: &str) -> io::Result<bool> {
    let data = fs::read_to_string("user.json")?;
    let user: UserRecord = serde_json::from_str(&data)?;

    if user.username != username {
        let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$\
                          AAAAAAAAAAAAAAAAAAAAAA$\
                          AAAAAAAAAAAAAAAAAAAA";
        let parsed = PasswordHash::new(dummy_hash).unwrap();
        let _ = Argon2::default().verify_password(b"dummy", &parsed);
        return Ok(false);
    }

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .expect("Stored hash is invalid");
    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
