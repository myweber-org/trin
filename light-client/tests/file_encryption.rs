use std::fs;
use std::io::{self, Read, Write};

fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

fn encrypt_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    xor_cipher(&mut buffer, key.as_bytes());
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    
    Ok(())
}

fn decrypt_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

fn main() -> io::Result<()> {
    let key = "secret_key";
    let original = "test_data.txt";
    let encrypted = "encrypted.bin";
    let decrypted = "decrypted.txt";
    
    fs::write(original, "Sensitive information here")?;
    
    encrypt_file(original, encrypted, key)?;
    println!("File encrypted successfully");
    
    decrypt_file(encrypted, decrypted, key)?;
    println!("File decrypted successfully");
    
    let restored = fs::read_to_string(decrypted)?;
    println!("Restored content: {}", restored);
    
    fs::remove_file(original)?;
    fs::remove_file(encrypted)?;
    fs::remove_file(decrypted)?;
    
    Ok(())
}