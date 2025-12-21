
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LEN],
    pub iv: [u8; IV_LEN],
}

pub fn encrypt_data(password: &str, plaintext: &[u8]) -> Result<EncryptionResult, String> {
    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut iv);
    
    let key = derive_key(password, &salt)?;
    
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    
    Ok(EncryptionResult {
        ciphertext,
        salt,
        iv,
    })
}

pub fn decrypt_data(password: &str, result: &EncryptionResult) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &result.salt)?;
    
    let decrypted = Aes256CbcDec::new(&key.into(), &result.iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&result.ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    Ok(decrypted)
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], String> {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        salt,
        PBKDF2_ITERATIONS,
        &mut key,
    );
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let password = "secure_password_123";
        let plaintext = b"Hello, this is a secret message!";
        
        let encrypted = encrypt_data(password, plaintext).unwrap();
        let decrypted = decrypt_data(password, &encrypted).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let plaintext = b"Test data";
        
        let encrypted = encrypt_data(password, plaintext).unwrap();
        let result = decrypt_data(wrong_password, &encrypted);
        
        assert!(result.is_err());
    }
}