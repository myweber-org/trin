use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::error::Error;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn generate_key() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut key = [0u8; 32];
    rng.fill(&mut key);
    key
}

pub fn generate_iv() -> [u8; 16] {
    let mut rng = rand::thread_rng();
    let mut iv = [0u8; 16];
    rng.fill(&mut iv);
    iv
}

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Result<String, Box<dyn Error>> {
    let ciphertext = Aes256CbcEnc::new(key.into(), iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    Ok(hex::encode(ciphertext))
}

pub fn decrypt_data(ciphertext_hex: &str, key: &[u8; 32], iv: &[u8; 16]) -> Result<Vec<u8>, Box<dyn Error>> {
    let ciphertext = hex::decode(ciphertext_hex)?;
    let plaintext = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let iv = generate_iv();
        let original_data = b"Secret message for encryption test";

        let encrypted = encrypt_data(original_data, &key, &iv).unwrap();
        let decrypted = decrypt_data(&encrypted, &key, &iv).unwrap();

        assert_eq!(original_data.to_vec(), decrypted);
    }
}