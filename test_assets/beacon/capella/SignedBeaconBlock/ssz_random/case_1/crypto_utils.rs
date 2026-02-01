use sha2::{Digest, Sha256};
use rand::{rngs::OsRng, RngCore};

pub fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; length];
    OsRng.fill_bytes(&mut buffer);
    buffer
}

pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn hash_to_hex_string(data: &[u8]) -> String {
    let hash = sha256_hash(data);
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bytes_length() {
        let bytes = generate_random_bytes(32);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_hash_consistency() {
        let input = b"test data";
        let hash1 = sha256_hash(input);
        let hash2 = sha256_hash(input);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hex_representation() {
        let input = b"hello";
        let hex_string = hash_to_hex_string(input);
        assert_eq!(hex_string.len(), 64);
        assert!(hex_string.chars().all(|c| c.is_ascii_hexdigit()));
    }
}