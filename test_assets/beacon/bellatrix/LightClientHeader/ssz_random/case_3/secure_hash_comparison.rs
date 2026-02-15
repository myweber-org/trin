use subtle::ConstantTimeEq;

pub fn verify_hash(secret_hash: &[u8], user_input: &[u8]) -> bool {
    if secret_hash.len() != user_input.len() {
        return false;
    }
    
    secret_hash.ct_eq(user_input).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_matching_hashes() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert!(verify_hash(&hash1, &hash2));
    }

    #[test]
    fn test_different_hashes() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("f3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert!(!verify_hash(&hash1, &hash2));
    }

    #[test]
    fn test_different_lengths() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("e3b0c442");
        assert!(!verify_hash(&hash1, &hash2));
    }
}