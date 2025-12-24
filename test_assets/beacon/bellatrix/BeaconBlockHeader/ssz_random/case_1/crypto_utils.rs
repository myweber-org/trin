
use argon2::{self, Config, ThreadMode, Variant, Version};
use rand::RngCore;

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

pub fn hash_password(password: &str) -> Result<String, argon2::Error> {
    let salt = generate_salt();
    let config = Config {
        variant: Variant::Argon2id,
        version: Version::Version13,
        mem_cost: 65536,
        time_cost: 3,
        lanes: 4,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    
    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &config)?;
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "SecurePass123!";
        let hash_result = hash_password(password);
        assert!(hash_result.is_ok());
        
        let hash = hash_result.unwrap();
        let verify_result = verify_password(password, &hash);
        assert_eq!(verify_result, Ok(true));
        
        let wrong_password = "WrongPass456!";
        let wrong_verify = verify_password(wrong_password, &hash);
        assert_eq!(wrong_verify, Ok(false));
    }

    #[test]
    fn test_salt_generation() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        assert_ne!(salt1, salt2);
        assert_eq!(salt1.len(), 32);
        assert_eq!(salt2.len(), 32);
    }
}