
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0xAA;

fn xor_cipher(data: &mut [u8], key: u8) {
    for byte in data {
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
    
    if args.len() != 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let mode = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];

    if !Path::new(input_file).exists() {
        eprintln!("Error: Input file '{}' not found", input_file);
        std::process::exit(1);
    }

    let key = match std::env::var("ENCRYPTION_KEY") {
        Ok(val) => val.parse().unwrap_or(DEFAULT_KEY),
        Err(_) => DEFAULT_KEY,
    };

    match mode.as_str() {
        "encrypt" | "decrypt" => process_file(input_file, output_file, key),
        _ => {
            eprintln!("Error: Mode must be 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
}