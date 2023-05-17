use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::Error as Argon2Error;
use argon2::password_hash::PasswordHash;
use argon2::password_hash::PasswordHasher;
use argon2::password_hash::PasswordVerifier;
use argon2::password_hash::SaltString;
use argon2::Argon2;
use thiserror::Error;

pub struct HashImpl;

// needed because argon2::password_hash::Error doesn't implement std::error::Error
#[derive(Error, Debug)]
pub enum HashImplError {
    #[error("argon2 error: {}", .0.to_string())]
    Argon2Error(Argon2Error),
}

impl From<Argon2Error> for HashImplError {
    fn from(err: Argon2Error) -> Self {
        Self::Argon2Error(err)
    }
}

impl HashImpl {
    #[allow(dead_code)]
    pub fn hash(password: &str) -> Result<String, HashImplError> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
        Ok(hash.to_string())
    }

    pub fn verify(password: &str, hash: &str) -> Result<(), HashImplError> {
        let parsed_hash = PasswordHash::new(hash)?;
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;
        Ok(())
    }
}
