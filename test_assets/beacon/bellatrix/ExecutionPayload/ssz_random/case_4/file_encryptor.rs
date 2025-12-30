use std::env;
use std::fs;

fn xor_cipher(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <key>", args[0]);
        std::process::exit(1);
    }

    let operation = &args[1];
    let input_path = &args[2];
    let key = args[3].as_bytes();

    let input_data = fs::read(input_path)?;

    let processed_data = xor_cipher(&input_data, key);

    let output_path = match operation.as_str() {
        "encrypt" => format!("{}.enc", input_path),
        "decrypt" => input_path.trim_end_matches(".enc").to_string(),
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    };

    fs::write(&output_path, processed_data)?;
    println!("Operation completed successfully. Output: {}", output_path);

    Ok(())
}