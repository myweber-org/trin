
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");

    let plaintext = fs::read(input_path)?;
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_data = key.to_vec();
    output_data.extend_from_slice(&ciphertext);
    fs::write(output_path, output_data)?;

    println!("File encrypted successfully. Key length: {} bytes", key.len());
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path)?;
    if data.len() < 32 {
        return Err("Invalid encrypted file format".into());
    }

    let (key_bytes, ciphertext) = data.split_at(32);
    let key = key_bytes.try_into()?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_12");

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, plaintext)?;
    println!("File decrypted successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Secret content for encryption test";
        let plain_file = NamedTempFile::new().unwrap();
        let enc_file = NamedTempFile::new().unwrap();
        let dec_file = NamedTempFile::new().unwrap();

        fs::write(plain_file.path(), test_data).unwrap();
        
        encrypt_file(
            plain_file.path().to_str().unwrap(),
            enc_file.path().to_str().unwrap()
        ).unwrap();
        
        decrypt_file(
            enc_file.path().to_str().unwrap(),
            dec_file.path().to_str().unwrap()
        ).unwrap();
        
        let decrypted = fs::read(dec_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key.as_bytes());
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let original = b"Hello, World!";
        let mut data = original.to_vec();
        
        xor_cipher(&mut data, key.as_bytes());
        assert_ne!(data.as_slice(), original);
        
        xor_cipher(&mut data, key.as_bytes());
        assert_eq!(data.as_slice(), original);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key_123";
        let mut input_file = NamedTempFile::new()?;
        write!(input_file, "Sensitive data: 42")?;
        
        let output_path = input_file.path().with_extension("enc");
        let decrypted_path = input_file.path().with_extension("dec");
        
        encrypt_file(input_file.path(), &output_path, key)?;
        decrypt_file(&output_path, &decrypted_path, key)?;
        
        let original = fs::read_to_string(input_file.path())?;
        let decrypted = fs::read_to_string(&decrypted_path)?;
        
        assert_eq!(original, decrypted);
        Ok(())
    }
}