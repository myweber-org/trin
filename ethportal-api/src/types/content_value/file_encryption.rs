
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LENGTH: usize = 32;

pub struct FileCipher;

impl FileCipher {
    pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext).map_err(|e| format!("Failed to read file: {}", e))?;

        let mut salt = [0u8; SALT_LENGTH];
        let mut iv = [0u8; IV_LENGTH];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);

        let key = Self::derive_key(password, &salt);

        let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut output = fs::File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
        output.write_all(&salt).map_err(|e| format!("Failed to write salt: {}", e))?;
        output.write_all(&iv).map_err(|e| format!("Failed to write IV: {}", e))?;
        output.write_all(&ciphertext).map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data).map_err(|e| format!("Failed to read file: {}", e))?;

        if encrypted_data.len() < SALT_LENGTH + IV_LENGTH {
            return Err("File too short to contain salt and IV".to_string());
        }

        let (salt, rest) = encrypted_data.split_at(SALT_LENGTH);
        let (iv, ciphertext) = rest.split_at(IV_LENGTH);

        let key = Self::derive_key(password, salt);

        let plaintext = Aes256CbcDec::new(&key.into(), iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output = fs::File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
        output.write_all(&plaintext).map_err(|e| format!("Failed to write plaintext: {}", e))?;

        Ok(())
    }

    fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
        let mut key = [0u8; KEY_LENGTH];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, KEY_ITERATIONS, &mut key);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Secret data that needs protection";
        let password = "strong_password_123";
        
        let input_path = "test_input.txt";
        let encrypted_path = "test_encrypted.bin";
        let decrypted_path = "test_decrypted.txt";

        fs::write(input_path, test_data).unwrap();

        FileCipher::encrypt_file(input_path, encrypted_path, password).unwrap();
        FileCipher::decrypt_file(encrypted_path, decrypted_path, password).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);

        fs::remove_file(input_path).unwrap();
        fs::remove_file(encrypted_path).unwrap();
        fs::remove_file(decrypted_path).unwrap();
    }
}