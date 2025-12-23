
use std::fs;
use std::io::{self, Read, Write};

const DEFAULT_KEY: u8 = 0x55;

fn xor_encrypt_decrypt(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

fn process_file(input_path: &str, output_path: &str, key: u8) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_encrypt_decrypt(&mut buffer, key);
    
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
        Err(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_xor_encryption() {
        let original = b"Hello, World!";
        let mut data = original.to_vec();
        let key = 0x42;
        
        xor_encrypt_decrypt(&mut data, key);
        assert_ne!(data, original);
        
        xor_encrypt_decrypt(&mut data, key);
        assert_eq!(data, original);
    }
    
    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let test_input = "test_input.txt";
        let test_output = "test_output.txt";
        let test_final = "test_final.txt";
        
        fs::write(test_input, "Test data for encryption")?;
        
        process_file(test_input, test_output, DEFAULT_KEY)?;
        process_file(test_output, test_final, DEFAULT_KEY)?;
        
        let original = fs::read_to_string(test_input)?;
        let decrypted = fs::read_to_string(test_final)?;
        
        assert_eq!(original, decrypted);
        
        fs::remove_file(test_input)?;
        fs::remove_file(test_output)?;
        fs::remove_file(test_final)?;
        
        Ok(())
    }
}