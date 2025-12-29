
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub struct FileEncryptor {
    key: [u8; 32],
    iv: [u8; 16],
}

impl FileEncryptor {
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut key);
        rand::thread_rng().fill_bytes(&mut iv);
        Self { key, iv }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let ciphertext = Aes256CbcEnc::new(&self.key.into(), &self.iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let mut file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        let plaintext = Aes256CbcDec::new(&self.key.into(), &self.iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }

    pub fn save_key_iv(&self, path: &str) -> Result<(), String> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.key);
        data.extend_from_slice(&self.iv);
        
        fs::write(path, &data)
            .map_err(|e| format!("Failed to save key/IV: {}", e))?;
        Ok(())
    }

    pub fn load_key_iv(path: &str) -> Result<Self, String> {
        let data = fs::read(path)
            .map_err(|e| format!("Failed to read key/IV file: {}", e))?;
        
        if data.len() != 48 {
            return Err("Invalid key/IV file format".to_string());
        }

        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];
        key.copy_from_slice(&data[0..32]);
        iv.copy_from_slice(&data[32..48]);

        Ok(Self { key, iv })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = FileEncryptor::new();
        
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();

        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}