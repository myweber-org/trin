use rand::Rng;

pub fn generate_password(length: usize, use_uppercase: bool, use_numbers: bool, use_symbols: bool) -> String {
    let mut charset = String::from("abcdefghijklmnopqrstuvwxyz");
    let mut rng = rand::thread_rng();

    if use_uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if use_numbers {
        charset.push_str("0123456789");
    }
    if use_symbols {
        charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
    }

    let charset_bytes: Vec<u8> = charset.into_bytes();
    let charset_len = charset_bytes.len();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_len);
            charset_bytes[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_password(12, true, true, true);
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_lowercase_only() {
        let password = generate_password(8, false, false, false);
        assert!(password.chars().all(|c| c.is_lowercase()));
    }

    #[test]
    fn test_character_variety() {
        let password = generate_password(20, true, true, true);
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_symbol = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
        
        assert!(has_upper);
        assert!(has_digit);
        assert!(has_symbol);
    }
}