use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    algorithm: CipherAlgorithm,
}

impl FileEncryptor {
    pub fn new(algorithm: CipherAlgorithm) -> Self {
        Self { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let (ciphertext, key, nonce) = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => self.encrypt_aes(&plaintext)?,
            CipherAlgorithm::ChaCha20Poly1305 => self.encrypt_chacha(&plaintext)?,
        };

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        let key_path = output_path.with_extension("key");
        let nonce_path = output_path.with_extension("nonce");
        
        fs::write(&key_path, key)
            .map_err(|e| format!("Failed to write key file: {}", e))?;
        fs::write(&nonce_path, nonce)
            .map_err(|e| format!("Failed to write nonce file: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, key_path: &Path, nonce_path: &Path, output_path: &Path) -> Result<(), String> {
        let ciphertext = fs::read(input_path)
            .map_err(|e| format!("Failed to read ciphertext: {}", e))?;
        
        let key = fs::read(key_path)
            .map_err(|e| format!("Failed to read key: {}", e))?;
        
        let nonce = fs::read(nonce_path)
            .map_err(|e| format!("Failed to read nonce: {}", e))?;

        let plaintext = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => self.decrypt_aes(&ciphertext, &key, &nonce)?,
            CipherAlgorithm::ChaCha20Poly1305 => self.decrypt_chacha(&ciphertext, &key, &nonce)?,
        };

        fs::write(output_path, plaintext)
            .map_err(|e| format!("Failed to write decrypted file: {}", e))?;

        Ok(())
    }

    fn encrypt_aes(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, plaintext)
            .map_err(|e| format!("AES encryption failed: {}", e))?;
        
        Ok((ciphertext, key.to_vec(), nonce.to_vec()))
    }

    fn decrypt_aes(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, String> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("AES decryption failed: {}", e))
    }

    fn encrypt_chacha(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, plaintext)
            .map_err(|e| format!("ChaCha20 encryption failed: {}", e))?;
        
        Ok((ciphertext, key.to_vec(), nonce.to_vec()))
    }

    fn decrypt_chacha(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, String> {
        let key = ChaChaKey::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaChaNonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("ChaCha20 decryption failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let test_data = b"Hello, this is a secret message!";
        
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let key_file = NamedTempFile::new().unwrap();
        let nonce_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        
        let key = fs::read(key_file.path()).unwrap();
        let nonce = fs::read(nonce_file.path()).unwrap();
        
        fs::write(key_file.path(), &key).unwrap();
        fs::write(nonce_file.path(), &nonce).unwrap();
        
        encryptor.decrypt_file(
            encrypted_file.path(),
            key_file.path(),
            nonce_file.path(),
            decrypted_file.path()
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::ChaCha20Poly1305);
        let test_data = b"Another secret message for ChaCha!";
        
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), test_data).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let key_file = NamedTempFile::new().unwrap();
        let nonce_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        
        let key = fs::read(key_file.path()).unwrap();
        let nonce = fs::read(nonce_file.path()).unwrap();
        
        fs::write(key_file.path(), &key).unwrap();
        fs::write(nonce_file.path(), &nonce).unwrap();
        
        encryptor.decrypt_file(
            encrypted_file.path(),
            key_file.path(),
            nonce_file.path(),
            decrypted_file.path()
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}