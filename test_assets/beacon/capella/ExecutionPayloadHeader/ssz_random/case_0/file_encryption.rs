
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &Path, dest_path: &Path) -> std::io::Result<()> {
        let mut source_file = File::open(source_path)?;
        let mut buffer = Vec::new();
        source_file.read_to_end(&mut buffer)?;

        let encrypted_data = self.xor_transform(&buffer);

        let mut dest_file = File::create(dest_path)?;
        dest_file.write_all(&encrypted_data)?;

        Ok(())
    }

    pub fn decrypt_file(&self, source_path: &Path, dest_path: &Path) -> std::io::Result<()> {
        self.encrypt_file(source_path, dest_path)
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

pub fn process_directory(
    cipher: &XorCipher,
    dir_path: &Path,
    operation: &str,
) -> std::io::Result<()> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let extension = path.extension().unwrap_or_default();
            let new_extension = match operation {
                "encrypt" => "enc",
                "decrypt" => "dec",
                _ => "enc",
            };

            let mut new_path = path.clone();
            new_path.set_extension(new_extension);

            match operation {
                "encrypt" => cipher.encrypt_file(&path, &new_path)?,
                "decrypt" => cipher.decrypt_file(&path, &new_path)?,
                _ => {}
            }

            println!("Processed: {:?} -> {:?}", path, new_path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let cipher = XorCipher::new("secret_key");
        let original_data = b"Hello, World!";
        let encrypted = cipher.xor_transform(original_data);
        let decrypted = cipher.xor_transform(&encrypted);

        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_file_encryption() -> std::io::Result<()> {
        let cipher = XorCipher::new("test_key");
        
        let mut source_file = NamedTempFile::new()?;
        source_file.write_all(b"Test file content")?;
        
        let mut encrypted_file = NamedTempFile::new()?;
        let encrypted_path = encrypted_file.path().with_extension("enc");
        
        cipher.encrypt_file(source_file.path(), &encrypted_path)?;
        
        let mut decrypted_file = NamedTempFile::new()?;
        let decrypted_path = decrypted_file.path().with_extension("dec");
        
        cipher.decrypt_file(&encrypted_path, &decrypted_path)?;
        
        let original_content = fs::read(source_file.path())?;
        let decrypted_content = fs::read(decrypted_path)?;
        
        assert_eq!(original_content, decrypted_content);
        
        Ok(())
    }
}