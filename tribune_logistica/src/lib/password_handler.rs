use argon2::{
    password_hash::{
        PasswordHash,
        PasswordVerifier
    },
    Argon2,
};
use std::fs;
use std::io;

use crate::models::{LoginRecord, UserRecord};


pub fn verify_login(login:LoginRecord) -> io::Result<bool> {
    let data = fs::read_to_string("config/user.json")?;
    let user: UserRecord = serde_json::from_str(&data)?;
    println!("read user_config, user: {}",user.username);

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
