
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output = fs::File::create(output_path)?;
    output.write_all(&ciphertext)?;
    output.write_all(nonce.as_slice())?;
    output.write_all(&key.as_slice()[..32])?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data.len() < NONCE_SIZE + 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short",
        ));
    }

    let ciphertext_len = data.len() - NONCE_SIZE - 32;
    let ciphertext = &data[..ciphertext_len];
    let nonce = &data[ciphertext_len..ciphertext_len + NONCE_SIZE];
    let key_bytes = &data[ciphertext_len + NONCE_SIZE..];

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output = fs::File::create(output_path)?;
    output.write_all(&plaintext)?;

    Ok(())
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    xor_cipher(&mut buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = DEFAULT_KEY;
    process_file(Path::new(input_path), Path::new(output_path), key)
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = DEFAULT_KEY;
    process_file(Path::new(input_path), Path::new(output_path), key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let original_data = b"Hello, World!";
        let mut encrypted_data = original_data.to_vec();
        let key = 0x55;

        xor_cipher(&mut encrypted_data, key);
        assert_ne!(encrypted_data.as_slice(), original_data);

        xor_cipher(&mut encrypted_data, key);
        assert_eq!(encrypted_data.as_slice(), original_data);
    }

    #[test]
    fn test_file_encryption_decryption() -> io::Result<()> {
        let original_content = b"Secret data for testing";
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(original_content)?;

        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;

        encrypt_file(input_file.path().to_str().unwrap(), encrypted_file.path().to_str().unwrap())?;
        decrypt_file(encrypted_file.path().to_str().unwrap(), decrypted_file.path().to_str().unwrap())?;

        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())?.read_to_end(&mut decrypted_content)?;

        assert_eq!(decrypted_content.as_slice(), original_content);
        Ok(())
    }
}