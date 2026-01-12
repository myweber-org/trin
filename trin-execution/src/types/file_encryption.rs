use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

fn xor_cipher(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

fn process_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let key_bytes = key.as_bytes();
    let mut content = fs::read(input_path)?;
    
    xor_cipher(&mut content, key_bytes);
    
    fs::write(output_path, content)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input> <output> <key>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    let key = &args[3];
    
    if !input_path.exists() {
        eprintln!("Error: Input file does not exist");
        std::process::exit(1);
    }
    
    process_file(input_path, output_path, key)?;
    println!("File processed successfully");
    Ok(())
}