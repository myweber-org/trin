use std::fs;
use std::io::{self, Read, Write};

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        self.process_file(input_path, output_path)
    }

    fn process_file(&self, input_path: &str, output_path: &str) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let processed_data: Vec<u8> = buffer
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect();

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&processed_data)?;

        Ok(())
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_cipher() {
        let test_data = b"Hello, World!";
        let key = "secret";
        let cipher = XorCipher::new(key);

        let temp_dir = std::env::temp_dir();
        let input_path = temp_dir.join("test_input.txt");
        let encrypted_path = temp_dir.join("test_encrypted.bin");
        let decrypted_path = temp_dir.join("test_decrypted.txt");

        fs::write(&input_path, test_data).unwrap();
        
        cipher.encrypt_file(input_path.to_str().unwrap(), 
                          encrypted_path.to_str().unwrap()).unwrap();
        cipher.decrypt_file(encrypted_path.to_str().unwrap(), 
                          decrypted_path.to_str().unwrap()).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(decrypted_data, test_data);

        fs::remove_file(input_path).ok();
        fs::remove_file(encrypted_path).ok();
        fs::remove_file(decrypted_path).ok();
    }
}