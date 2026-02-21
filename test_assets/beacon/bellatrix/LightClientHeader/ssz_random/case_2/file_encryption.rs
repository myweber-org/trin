use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;
    Ok((ciphertext, nonce.to_vec()))
}

pub fn decrypt_data(ciphertext: &[u8], nonce: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"Secret message for encryption test";
        let (ciphertext, nonce) = encrypt_data(original_data).unwrap();
        
        let key = Aes256Gcm::generate_key(&mut OsRng).to_vec();
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let encrypted = cipher.encrypt(Nonce::from_slice(&nonce), original_data).unwrap();
        
        let decrypted = decrypt_data(&encrypted, &nonce, &key).unwrap();
        assert_eq!(original_data.to_vec(), decrypted);
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &Path, dest_path: &Path) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut dest_file = fs::File::create(dest_path)?;

        let mut buffer = [0; 4096];
        let mut key_index = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let mut processed = Vec::with_capacity(bytes_read);
            for i in 0..bytes_read {
                let byte = buffer[i] ^ self.key[key_index];
                processed.push(byte);
                key_index = (key_index + 1) % self.key.len();
            }

            dest_file.write_all(&processed)?;
        }

        dest_file.flush()?;
        Ok(())
    }

    pub fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.process_data(data)
    }

    pub fn decrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.process_data(data)
    }

    fn process_data(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        let mut key_index = 0;

        for &byte in data {
            result.push(byte ^ self.key[key_index]);
            key_index = (key_index + 1) % self.key.len();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XORCipher::new("secret_key");
        let original_data = b"Hello, World! This is test data.";
        
        let encrypted = cipher.encrypt_data(original_data);
        let decrypted = cipher.decrypt_data(&encrypted);
        
        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let cipher = XORCipher::new("test_key_123");
        
        let original_content = b"Sample file content for encryption test.";
        let mut source_file = NamedTempFile::new()?;
        source_file.write_all(original_content)?;
        
        let encrypted_file = NamedTempFile::new()?;
        cipher.encrypt_file(source_file.path(), encrypted_file.path())?;
        
        let decrypted_file = NamedTempFile::new()?;
        cipher.decrypt_file(encrypted_file.path(), decrypted_file.path())?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())?.read_to_end(&mut decrypted_content)?;
        
        assert_eq!(original_content, decrypted_content.as_slice());
        Ok(())
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
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

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let mut input_file = File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let (ciphertext, key, nonce) = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                let key = Aes256Gcm::generate_key(&mut OsRng);
                let cipher = Aes256Gcm::new(&key);
                let nonce = Nonce::from_slice(&[0u8; 12]);
                let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
                    .map_err(|e| format!("Encryption failed: {}", e))?;
                (ciphertext, key.to_vec(), nonce.to_vec())
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                let cipher = ChaCha20Poly1305::new(&key);
                let nonce = ChaChaNonce::from_slice(&[0u8; 12]);
                let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
                    .map_err(|e| format!("Encryption failed: {}", e))?;
                (ciphertext, key.to_vec(), nonce.to_vec())
            }
        };

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&ciphertext)?;

        let key_path = format!("{}.key", output_path);
        let mut key_file = File::create(&key_path)?;
        key_file.write_all(&key)?;
        key_file.write_all(&nonce)?;

        println!("Encryption completed. Key saved to: {}", key_path);
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let mut encrypted_file = File::open(input_path)?;
        let mut ciphertext = Vec::new();
        encrypted_file.read_to_end(&mut ciphertext)?;

        let mut key_file = File::open(key_path)?;
        let mut key_nonce = Vec::new();
        key_file.read_to_end(&mut key_nonce)?;

        let (key_bytes, nonce_bytes) = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                if key_nonce.len() != 44 {
                    return Err("Invalid key file length".into());
                }
                (&key_nonce[0..32], &key_nonce[32..44])
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                if key_nonce.len() != 44 {
                    return Err("Invalid key file length".into());
                }
                (&key_nonce[0..32], &key_nonce[32..44])
            }
        };

        let plaintext = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                let key = Key::<Aes256Gcm>::from_slice(key_bytes);
                let cipher = Aes256Gcm::new(key);
                let nonce = Nonce::from_slice(nonce_bytes);
                cipher.decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| format!("Decryption failed: {}", e))?
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                let key = ChaChaKey::from_slice(key_bytes);
                let cipher = ChaCha20Poly1305::new(key);
                let nonce = ChaChaNonce::from_slice(nonce_bytes);
                cipher.decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| format!("Decryption failed: {}", e))?
            }
        };

        let mut output_file = File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        println!("Decryption completed successfully.");
        Ok(())
    }
}

pub fn generate_random_key() -> Vec<u8> {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let test_data = b"Test encryption data for AES-256-GCM";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let key_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();

        let key_path = format!("{}.key", encrypted_file.path().to_str().unwrap());
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            &key_path,
            decrypted_file.path().to_str().unwrap()
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::ChaCha20Poly1305);
        let test_data = b"Test encryption data for ChaCha20Poly1305";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();

        let key_path = format!("{}.key", encrypted_file.path().to_str().unwrap());
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            &key_path,
            decrypted_file.path().to_str().unwrap()
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}