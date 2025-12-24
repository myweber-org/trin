
use rand::rngs::OsRng;
use rand::RngCore;
use std::fmt::Write;

const PASSWORD_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                 abcdefghijklmnopqrstuvwxyz\
                                 0123456789\
                                 !@#$%^&*()_+-=[]{}|;:,.<>?";

pub fn generate_secure_password(length: usize) -> Result<String, &'static str> {
    if length < 8 {
        return Err("Password length must be at least 8 characters");
    }
    
    let mut rng = OsRng;
    let mut password = String::with_capacity(length);
    
    for _ in 0..length {
        let mut random_bytes = [0u8; 4];
        rng.fill_bytes(&mut random_bytes);
        
        let index = u32::from_le_bytes(random_bytes) as usize % PASSWORD_CHARSET.len();
        password.push(PASSWORD_CHARSET[index] as char);
    }
    
    Ok(password)
}

pub fn validate_password_strength(password: &str) -> bool {
    if password.len() < 8 {
        return false;
    }
    
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| 
        PASSWORD_CHARSET[62..].contains(&(c as u8))
    );
    
    has_upper && has_lower && has_digit && has_special
}