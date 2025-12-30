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
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let (ciphertext, key, nonce) = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                let key = Aes256Gcm::generate_key(&mut OsRng);
                let cipher = Aes256Gcm::new(&key);
                let nonce = Nonce::from_slice(b"unique_nonce_");
                let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
                    .map_err(|e| format!("AES encryption failed: {}", e))?;
                (ciphertext, key.as_slice().to_vec(), nonce.to_vec())
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                let cipher = ChaCha20Poly1305::new(&key);
                let nonce = ChaChaNonce::from_slice(b"unique_nonce_");
                let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
                    .map_err(|e| format!("ChaCha20 encryption failed: {}", e))?;
                (ciphertext, key.as_slice().to_vec(), nonce.to_vec())
            }
        };

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        let metadata = EncryptedMetadata {
            algorithm: self.algorithm,
            key,
            nonce,
        };
        
        let serialized_meta = bincode::serialize(&metadata)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
        
        output_file.write_all(&(serialized_meta.len() as u32).to_le_bytes())
            .map_err(|e| format!("Failed to write metadata length: {}", e))?;
        output_file.write_all(&serialized_meta)
            .map_err(|e| format!("Failed to write metadata: {}", e))?;
        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut meta_len_bytes = [0u8; 4];
        file.read_exact(&mut meta_len_bytes)
            .map_err(|e| format!("Failed to read metadata length: {}", e))?;
        let meta_len = u32::from_le_bytes(meta_len_bytes) as usize;
        
        let mut meta_buf = vec![0u8; meta_len];
        file.read_exact(&mut meta_buf)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        
        let metadata: EncryptedMetadata = bincode::deserialize(&meta_buf)
            .map_err(|e| format!("Failed to deserialize metadata: {}", e))?;
        
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read ciphertext: {}", e))?;

        let plaintext = match metadata.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                let key = Key::<Aes256Gcm>::from_slice(&metadata.key);
                let cipher = Aes256Gcm::new(key);
                let nonce = Nonce::from_slice(&metadata.nonce);
                cipher.decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| format!("AES decryption failed: {}", e))?
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                let key = ChaChaKey::from_slice(&metadata.key);
                let cipher = ChaCha20Poly1305::new(key);
                let nonce = ChaChaNonce::from_slice(&metadata.nonce);
                cipher.decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| format!("ChaCha20 decryption failed: {}", e))?
            }
        };

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write plaintext: {}", e))?;

        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EncryptedMetadata {
    algorithm: CipherAlgorithm,
    key: Vec<u8>,
    nonce: Vec<u8>,
}