use rand::rngs::OsRng;
use rand::RngCore;

const PASSWORD_LENGTH: usize = 16;
const CHARACTER_SET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                               abcdefghijklmnopqrstuvwxyz\
                               0123456789\
                               !@#$%^&*()-_=+[]{}|;:,.<>?";

pub fn generate_secure_password() -> String {
    let mut rng = OsRng;
    let mut password_bytes = vec![0u8; PASSWORD_LENGTH];
    
    for byte in password_bytes.iter_mut() {
        let random_index = (rng.next_u32() as usize) % CHARACTER_SET.len();
        *byte = CHARACTER_SET[random_index];
    }
    
    String::from_utf8(password_bytes).expect("Generated valid UTF-8 password")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_secure_password();
        assert_eq!(password.len(), PASSWORD_LENGTH);
    }

    #[test]
    fn test_password_charset() {
        let password = generate_secure_password();
        for ch in password.chars() {
            assert!(CHARACTER_SET.contains(&(ch as u8)));
        }
    }

    #[test]
    fn test_unique_passwords() {
        let pass1 = generate_secure_password();
        let pass2 = generate_secure_password();
        assert_ne!(pass1, pass2);
    }
}