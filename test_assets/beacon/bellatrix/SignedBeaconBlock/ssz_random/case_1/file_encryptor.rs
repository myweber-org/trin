use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer.iter().map(|&byte| byte ^ encryption_key).collect();

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_directory(dir_path: &str, key: Option<u8>, encrypt: bool) -> io::Result<()> {
    let dir = Path::new(dir_path);
    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Provided path is not a directory",
        ));
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let input_str = path.to_str().unwrap();
            let output_str = if encrypt {
                format!("{}.enc", input_str)
            } else {
                input_str.trim_end_matches(".enc").to_string()
            };

            if encrypt {
                encrypt_file(input_str, &output_str, key)?;
                println!("Encrypted: {} -> {}", input_str, output_str);
            } else {
                decrypt_file(input_str, &output_str, key)?;
                println!("Decrypted: {} -> {}", input_str, output_str);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, World!";
        let test_file = "test_input.txt";
        let encrypted_file = "test_encrypted.enc";
        let decrypted_file = "test_decrypted.txt";

        fs::write(test_file, test_data).unwrap();
        encrypt_file(test_file, encrypted_file, Some(0xAA)).unwrap();
        decrypt_file(encrypted_file, decrypted_file, Some(0xAA)).unwrap();

        let decrypted_content = fs::read(decrypted_file).unwrap();
        assert_eq!(decrypted_content, test_data);

        fs::remove_file(test_file).unwrap();
        fs::remove_file(encrypted_file).unwrap();
        fs::remove_file(decrypted_file).unwrap();
    }
}