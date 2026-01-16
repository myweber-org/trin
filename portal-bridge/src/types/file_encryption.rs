
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::{pbkdf2_hmac, Params};
use rand::RngCore;
use sha2::Sha256;
use std::fs::{self, File};
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct EncryptionResult {
    pub salt: [u8; SALT_LEN],
    pub iv: [u8; IV_LEN],
    pub ciphertext: Vec<u8>,
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut input_file = File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        &salt,
        PBKDF2_ITERATIONS,
        &mut key,
        Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: KEY_LEN,
        },
    );

    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&ciphertext)?;

    Ok(EncryptionResult {
        salt,
        iv,
        ciphertext,
    })
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
    salt: &[u8; SALT_LEN],
    iv: &[u8; IV_LEN],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut input_file = File::open(input_path)?;
    let mut ciphertext = Vec::new();
    input_file.read_to_end(&mut ciphertext)?;

    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        salt,
        PBKDF2_ITERATIONS,
        &mut key,
        Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: KEY_LEN,
        },
    );

    let plaintext = Aes256CbcDec::new(&key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}

pub fn generate_key_file(password: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);

    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        &salt,
        PBKDF2_ITERATIONS,
        &mut key,
        Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: KEY_LEN,
        },
    );

    let mut key_file = File::create(output_path)?;
    key_file.write_all(&salt)?;
    key_file.write_all(&key)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Secret data that needs protection";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), plaintext).unwrap();

        let result = encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            password,
        ).unwrap();

        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            password,
            &result.salt,
            &result.iv,
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted_data);
    }

    #[test]
    fn test_key_generation() {
        let password = "test_password";
        let key_file = NamedTempFile::new().unwrap();

        generate_key_file(password, key_file.path().to_str().unwrap()).unwrap();

        let key_data = fs::read(key_file.path()).unwrap();
        assert_eq!(key_data.len(), SALT_LEN + KEY_LEN);
    }
}