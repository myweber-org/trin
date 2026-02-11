use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;

fn main() {
    println!("Secure Password Generator");
    println!("=========================");
    
    let length = get_password_length();
    let include_uppercase = get_yes_no_input("Include uppercase letters? (y/n): ");
    let include_lowercase = get_yes_no_input("Include lowercase letters? (y/n): ");
    let include_numbers = get_yes_no_input("Include numbers? (y/n): ");
    let include_symbols = get_yes_no_input("Include symbols? (y/n): ");

    let password = generate_password(length, include_uppercase, include_lowercase, include_numbers, include_symbols);
    
    println!("\nGenerated Password: {}", password);
    println!("Password Strength: {}", evaluate_password_strength(&password));
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (default: {}): ", DEFAULT_LENGTH);
        let mut input = String::new();
        
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
            
        let input = input.trim();
        
        if input.is_empty() {
            return DEFAULT_LENGTH;
        }
        
        match input.parse::<usize>() {
            Ok(length) if length >= 4 && length <= 128 => return length,
            Ok(_) => println!("Password length must be between 4 and 128 characters"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn get_yes_no_input(prompt: &str) -> bool {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
            
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please enter 'y' or 'n'"),
        }
    }
}

fn generate_password(
    length: usize,
    uppercase: bool,
    lowercase: bool,
    numbers: bool,
    symbols: bool,
) -> String {
    let mut character_pool = Vec::new();
    
    if uppercase {
        character_pool.extend(b'A'..=b'Z');
    }
    if lowercase {
        character_pool.extend(b'a'..=b'z');
    }
    if numbers {
        character_pool.extend(b'0'..=b'9');
    }
    if symbols {
        character_pool.extend(b'!'..=b'/');
        character_pool.extend(b':'..=b'@');
        character_pool.extend(b'['..=b'`');
        character_pool.extend(b'{'..=b'~');
    }
    
    if character_pool.is_empty() {
        character_pool.extend(b'a'..=b'z');
    }
    
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);
    
    for _ in 0..length {
        let idx = rng.gen_range(0..character_pool.len());
        password.push(character_pool[idx] as char);
    }
    
    password
}

fn evaluate_password_strength(password: &str) -> String {
    let length = password.len();
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());
    
    let mut score = 0;
    
    if length >= 12 { score += 2; }
    else if length >= 8 { score += 1; }
    
    if has_upper { score += 1; }
    if has_lower { score += 1; }
    if has_digit { score += 1; }
    if has_symbol { score += 1; }
    
    match score {
        0..=2 => "Weak",
        3..=4 => "Medium",
        5..=6 => "Strong",
        _ => "Very Strong",
    }.to_string()
}