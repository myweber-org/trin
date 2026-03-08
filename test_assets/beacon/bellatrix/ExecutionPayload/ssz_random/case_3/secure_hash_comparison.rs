use subtle::{Choice, ConstantTimeEq};

pub fn compare_hashes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = Choice::from(1);
    for (x, y) in a.iter().zip(b.iter()) {
        result &= x.ct_eq(y);
    }
    
    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_matching_hashes() {
        let hash1 = hex!("d41d8cd98f00b204e9800998ecf8427e");
        let hash2 = hex!("d41d8cd98f00b204e9800998ecf8427e");
        assert!(compare_hashes(&hash1, &hash2));
    }

    #[test]
    fn test_different_hashes() {
        let hash1 = hex!("d41d8cd98f00b204e9800998ecf8427e");
        let hash2 = hex!("c4ca4238a0b923820dcc509a6f75849b");
        assert!(!compare_hashes(&hash1, &hash2));
    }

    #[test]
    fn test_different_lengths() {
        let hash1 = hex!("d41d8cd98f00b204e9800998ecf8427e");
        let hash2 = hex!("d41d8cd98f00b204e9800998ecf842");
        assert!(!compare_hashes(&hash1, &hash2));
    }
}