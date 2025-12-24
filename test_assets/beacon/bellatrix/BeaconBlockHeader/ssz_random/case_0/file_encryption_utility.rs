use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
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
    
    match process_file(input_file, output_file, DEFAULT_KEY) {
        Ok(_) => println!("File processed successfully"),
        Err(e) => eprintln!("Error processing file: {}", e),
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
        let mut data = vec![0x00, 0xFF, 0x55, 0xAA];
        let original = data.clone();
        let key = 0xAA;
        
        xor_cipher(&mut data, key);
        xor_cipher(&mut data, key);
        
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_processing() -> io::Result<()> {
        let mut input_file = NamedTempFile::new()?;
        let test_data = b"Test data for encryption";
        input_file.write_all(test_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        process_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            DEFAULT_KEY
        )?;
        
        let mut processed_data = Vec::new();
        fs::File::open(output_file.path())?
            .read_to_end(&mut processed_data)?;
        
        assert_ne!(processed_data, test_data);
        
        let mut temp_file = NamedTempFile::new()?;
        process_file(
            output_file.path().to_str().unwrap(),
            temp_file.path().to_str().unwrap(),
            DEFAULT_KEY
        )?;
        
        let mut restored_data = Vec::new();
        fs::File::open(temp_file.path())?
            .read_to_end(&mut restored_data)?;
        
        assert_eq!(restored_data, test_data);
        
        Ok(())
    }
}