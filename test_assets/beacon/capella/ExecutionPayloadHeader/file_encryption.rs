use std::fs;
use std::io::{self, Read, Write};

const KEY: u8 = 0xAA;

fn xor_encrypt_decrypt(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|&byte| byte ^ key).collect()
}

fn process_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let processed_data = xor_encrypt_decrypt(&buffer, KEY);

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

    process_file(input_file, output_file)?;
    println!("File processed successfully: {} -> {}", input_file, output_file);

    Ok(())
}