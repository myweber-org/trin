
use rand::{thread_rng, RngCore};
use std::fmt;

const KEY_LENGTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionKey {
    bytes: [u8; KEY_LENGTH],
}

impl EncryptionKey {
    pub fn generate() -> Self {
        let mut rng = thread_rng();
        let mut key_bytes = [0u8; KEY_LENGTH];
        rng.fill_bytes(&mut key_bytes);
        Self { bytes: key_bytes }
    }

    pub fn as_bytes(&self) -> &[u8; KEY_LENGTH] {
        &self.bytes
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.bytes)
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != KEY_LENGTH {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; KEY_LENGTH];
        array.copy_from_slice(&bytes);
        Ok(Self { bytes: array })
    }
}

impl fmt::Display for EncryptionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl AsRef<[u8]> for EncryptionKey {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key1 = EncryptionKey::generate();
        let key2 = EncryptionKey::generate();
        assert_ne!(key1, key2);
        assert_eq!(key1.as_bytes().len(), KEY_LENGTH);
    }

    #[test]
    fn test_hex_conversion() {
        let key = EncryptionKey::generate();
        let hex_str = key.to_hex();
        let restored_key = EncryptionKey::from_hex(&hex_str).unwrap();
        assert_eq!(key, restored_key);
    }

    #[test]
    fn test_invalid_hex() {
        let result = EncryptionKey::from_hex("invalid");
        assert!(result.is_err());
    }
}