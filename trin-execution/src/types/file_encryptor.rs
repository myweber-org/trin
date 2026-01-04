use std::fs;
use std::io::{self, Read, Write};
use base64::{Engine as _, engine::general_purpose};

fn xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn write_file(path: &str, data: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(data)
}

fn encrypt_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    let data = read_file(input_path)?;
    let encrypted = xor_cipher(&data, key.as_bytes());
    let encoded = general_purpose::STANDARD.encode(encrypted);
    write_file(output_path, encoded.as_bytes())
}

fn decrypt_file(input_path: &str, output_path: &str, key: &str) -> io::Result<()> {
    let encoded = read_file(input_path)?;
    let encoded_str = String::from_utf8_lossy(&encoded);
    let encrypted = general_purpose::STANDARD.decode(encoded_str.trim())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let decrypted = xor_cipher(&encrypted, key.as_bytes());
    write_file(output_path, &decrypted)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} <encrypt|decrypt> <input> <output> <key>", args[0]);
        std::process::exit(1);
    }

    let operation = &args[1];
    let input = &args[2];
    let output = &args[3];
    let key = &args[4];

    match operation.as_str() {
        "encrypt" => encrypt_file(input, output, key),
        "decrypt" => decrypt_file(input, output, key),
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let data = fs::read(input_path)?;
    
    let encrypted_data: Vec<u8> = data.iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input> <output> [key]", args[0]);
        std::process::exit(1);
    }
    
    let operation = &args[1];
    let input = &args[2];
    let output = &args[3];
    let key = args.get(4).and_then(|k| k.parse::<u8>().ok());
    
    if !Path::new(input).exists() {
        eprintln!("Error: Input file '{}' not found", input);
        std::process::exit(1);
    }
    
    match operation.as_str() {
        "encrypt" => {
            if let Err(e) = encrypt_file(input, output, key) {
                eprintln!("Encryption failed: {}", e);
                std::process::exit(1);
            }
            println!("File encrypted successfully");
        }
        "decrypt" => {
            if let Err(e) = decrypt_file(input, output, key) {
                eprintln!("Decryption failed: {}", e);
                std::process::exit(1);
            }
            println!("File decrypted successfully");
        }
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
}