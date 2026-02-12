
use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0x55;

fn xor_encrypt_decrypt(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|&byte| byte ^ key).collect()
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_encrypt_decrypt(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];

    println!("Processing file: {} -> {}", input_file, output_file);
    println!("Using XOR key: 0x{:02X}", DEFAULT_KEY);

    process_file(input_file, output_file, DEFAULT_KEY)?;
    
    println!("Operation completed successfully.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encrypt_decrypt() {
        let data = b"Hello, World!";
        let key = 0x42;
        
        let encrypted = xor_encrypt_decrypt(data, key);
        let decrypted = xor_encrypt_decrypt(&encrypted, key);
        
        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_file_processing() -> io::Result<()> {
        let mut input_temp = NamedTempFile::new()?;
        let test_data = b"Test file content";
        input_temp.write_all(test_data)?;
        
        let output_temp = NamedTempFile::new()?;
        
        process_file(
            input_temp.path().to_str().unwrap(),
            output_temp.path().to_str().unwrap(),
            DEFAULT_KEY
        )?;
        
        let mut output_content = Vec::new();
        fs::File::open(output_temp.path())?.read_to_end(&mut output_content)?;
        
        let decrypted = xor_encrypt_decrypt(&output_content, DEFAULT_KEY);
        assert_eq!(test_data, decrypted.as_slice());
        
        Ok(())
    }
}