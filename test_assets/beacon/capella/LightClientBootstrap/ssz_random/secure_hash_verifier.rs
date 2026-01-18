use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use std::error::Error;

type HmacSha256 = Hmac<Sha256>;

pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn verify_hmac(key: &[u8], data: &[u8], expected_mac: &str) -> Result<bool, Box<dyn Error>> {
    let mut mac = HmacSha256::new_from_slice(key)?;
    mac.update(data);
    let result = mac.verify_slice(&hex::decode(expected_mac)?)?;
    Ok(result.is_ok())
}

pub fn generate_hmac(key: &[u8], data: &[u8]) -> Result<String, Box<dyn Error>> {
    let mut mac = HmacSha256::new_from_slice(key)?;
    mac.update(data);
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_consistency() {
        let data = b"integrity check";
        let hash1 = compute_sha256(data);
        let hash2 = compute_sha256(data);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_hmac_verification() -> Result<(), Box<dyn Error>> {
        let key = b"secret-key";
        let message = b"sensitive data";
        
        let mac = generate_hmac(key, message)?;
        assert!(verify_hmac(key, message, &mac)?);
        
        let wrong_key = b"wrong-key";
        assert!(!verify_hmac(wrong_key, message, &mac)?);
        
        Ok(())
    }
}