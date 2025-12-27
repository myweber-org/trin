
use std::fs;
use std::io::{Read, Write};
use std::env;

fn xor_data(data: &mut [u8], key: &[u8]) {
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input_file> <output_file> <key>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let key = args[3].as_bytes();

    let mut content = fs::read(input_path)?;
    xor_data(&mut content, key);
    fs::write(output_path, &content)?;

    println!("File processed successfully: {} -> {}", input_path, output_path);
    Ok(())
}