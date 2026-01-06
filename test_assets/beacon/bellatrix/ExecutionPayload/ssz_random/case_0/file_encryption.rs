
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct FileCipher {
    cipher: Aes256Gcm,
}

impl FileCipher {
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let nonce = Nonce::from_slice(b"unique_nonce_");
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&ciphertext)?;
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut ciphertext = Vec::new();
        input_file.read_to_end(&mut ciphertext)?;

        let nonce = Nonce::from_slice(b"unique_nonce_");
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;
        Ok(())
    }
}

pub fn process_encryption() -> io::Result<()> {
    let cipher = FileCipher::new();
    let test_data = Path::new("test_data.bin");
    let encrypted = Path::new("encrypted.bin");
    let decrypted = Path::new("decrypted.bin");

    fs::write(test_data, b"Sample secret data")?;
    cipher.encrypt_file(test_data, encrypted)?;
    cipher.decrypt_file(encrypted, decrypted)?;

    let restored = fs::read_to_string(decrypted)?;
    assert_eq!(restored, "Sample secret data");

    fs::remove_file(test_data)?;
    fs::remove_file(encrypted)?;
    fs::remove_file(decrypted)?;
    Ok(())
}