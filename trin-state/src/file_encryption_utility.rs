use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let original_data = b"Test data for encryption";
        temp_file.write_all(original_data).unwrap();

        let input_path = temp_file.path().to_str().unwrap();
        let encrypted_path = format!("{}.enc", input_path);
        let decrypted_path = format!("{}.dec", input_path);

        encrypt_file(input_path, &encrypted_path, Some(0xAA)).unwrap();
        decrypt_file(&encrypted_path, &decrypted_path, Some(0xAA)).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(original_data, decrypted_data.as_slice());
    }
}