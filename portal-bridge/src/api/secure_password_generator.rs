use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
    let mut rng = thread_rng();
    let password: String = (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    password
}

pub fn generate_secure_password(length: usize) -> String {
    if length < 8 {
        panic!("Password length must be at least 8 characters for security.");
    }
    generate_password(length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password_length() {
        let password = generate_password(12);
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_generate_secure_password_valid() {
        let password = generate_secure_password(10);
        assert_eq!(password.len(), 10);
    }

    #[test]
    #[should_panic(expected = "Password length must be at least 8 characters for security.")]
    fn test_generate_secure_password_invalid() {
        generate_secure_password(5);
    }
}