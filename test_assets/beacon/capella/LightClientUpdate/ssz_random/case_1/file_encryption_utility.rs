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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_cipher_symmetry() {
        let cipher = XorCipher::new("secret_key");
        let test_data = b"Hello, World!";
        let test_file = "test_input.txt";
        let encrypted_file = "test_encrypted.bin";
        let decrypted_file = "test_decrypted.txt";

        fs::write(test_file, test_data).unwrap();
        cipher.encrypt_file(test_file, encrypted_file).unwrap();
        cipher.decrypt_file(encrypted_file, decrypted_file).unwrap();

        let decrypted_content = fs::read(decrypted_file).unwrap();
        assert_eq!(decrypted_content, test_data);

        fs::remove_file(test_file).unwrap_or_default();
        fs::remove_file(encrypted_file).unwrap_or_default();
        fs::remove_file(decrypted_file).unwrap_or_default();
    }
}