use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
    let mut rng = thread_rng();
    let password: String = (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    password
}

pub fn generate_secure_password(length: usize) -> Result<String, Box<dyn std::error::Error>> {
    if length < 8 {
        return Err("Password length must be at least 8 characters".into());
    }
    
    let mut rng = thread_rng();
    let mut password_bytes = vec![0u8; length];
    rng.fill(&mut password_bytes[..]);
    
    let password = password_bytes
        .iter()
        .map(|byte| {
            let idx = (byte % 62) as usize;
            let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            chars.chars().nth(idx).unwrap()
        })
        .collect();
    
    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_password(12);
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_secure_password_length() {
        let password = generate_secure_password(16).unwrap();
        assert_eq!(password.len(), 16);
    }

    #[test]
    fn test_secure_password_too_short() {
        let result = generate_secure_password(6);
        assert!(result.is_err());
    }
}