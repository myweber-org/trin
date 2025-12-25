use subtle::ConstantTimeEq;

pub fn compare_hashes(a: &[u8], b: &[u8]) -> bool {
    a.ct_eq(b).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_equal_hashes() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert!(compare_hashes(&hash1, &hash2));
    }

    #[test]
    fn test_different_hashes() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("d3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert!(!compare_hashes(&hash1, &hash2));
    }

    #[test]
    fn test_different_lengths() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85");
        assert!(!compare_hashes(&hash1, &hash2));
    }
}