
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex::{decode, encode};
use rand::RngCore;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn generate_random_iv() -> [u8; 16] {
    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);
    iv
}

pub fn encrypt_aes256_cbc(plaintext: &str, key: &[u8; 32], iv: &[u8; 16]) -> Result<String, String> {
    let plaintext_bytes = plaintext.as_bytes();
    let buffer_len = plaintext_bytes.len() + 16;
    let mut buffer = vec![0u8; buffer_len];

    let cipher = Aes256CbcEnc::new(key.into(), iv.into());
    let encrypted_len = cipher
        .encrypt_padded_b2b_mut::<Pkcs7>(plaintext_bytes, &mut buffer)
        .map_err(|e| format!("Encryption failed: {}", e))?
        .len();

    buffer.truncate(encrypted_len);
    Ok(encode(buffer))
}

pub fn decrypt_aes256_cbc(ciphertext_hex: &str, key: &[u8; 32], iv: &[u8; 16]) -> Result<String, String> {
    let ciphertext_bytes = decode(ciphertext_hex).map_err(|e| format!("Hex decode failed: {}", e))?;
    let buffer_len = ciphertext_bytes.len();
    let mut buffer = vec![0u8; buffer_len];

    let cipher = Aes256CbcDec::new(key.into(), iv.into());
    let decrypted_bytes = cipher
        .decrypt_padded_b2b_mut::<Pkcs7>(&ciphertext_bytes, &mut buffer)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(decrypted_bytes.to_vec()).map_err(|e| format!("UTF-8 conversion failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_random_key();
        let iv = generate_random_iv();
        let original_message = "Sensitive data requiring protection";

        let encrypted = encrypt_aes256_cbc(original_message, &key, &iv).unwrap();
        let decrypted = decrypt_aes256_cbc(&encrypted, &key, &iv).unwrap();

        assert_eq!(original_message, decrypted);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = [0u8; 16];
        let iv = generate_random_iv();
        let result = encrypt_aes256_cbc("test", &short_key, &iv);
        assert!(result.is_err());
    }
}