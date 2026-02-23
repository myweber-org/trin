use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn main() {
    println!("Secure Password Generator");
    println!("==========================");
    
    let length = get_password_length();
    let char_sets = select_character_sets();
    
    let password = generate_password(length, &char_sets);
    println!("\nGenerated Password: {}", password);
    println!("Password Strength: {}", assess_password_strength(&password));
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (default: {}):", DEFAULT_LENGTH);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return DEFAULT_LENGTH;
        }
        
        match trimmed.parse::<usize>() {
            Ok(length) if length >= 8 && length <= 128 => return length,
            Ok(_) => println!("Password length must be between 8 and 128 characters"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn select_character_sets() -> Vec<String> {
    let mut char_sets = Vec::new();
    let mut include_uppercase = true;
    let mut include_lowercase = true;
    let mut include_digits = true;
    let mut include_special = true;
    
    println!("\nSelect character sets to include:");
    println!("1. Uppercase letters: {}", if include_uppercase { "✓" } else { "✗" });
    println!("2. Lowercase letters: {}", if include_lowercase { "✓" } else { "✗" });
    println!("3. Digits: {}", if include_digits { "✓" } else { "✗" });
    println!("4. Special characters: {}", if include_special { "✓" } else { "✗" });
    println!("5. Generate password");
    
    loop {
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read line");
        
        match choice.trim() {
            "1" => include_uppercase = !include_uppercase,
            "2" => include_lowercase = !include_lowercase,
            "3" => include_digits = !include_digits,
            "4" => include_special = !include_special,
            "5" => break,
            _ => {
                println!("Invalid choice. Please enter 1-5");
                continue;
            }
        }
        
        println!("\nSelect character sets to include:");
        println!("1. Uppercase letters: {}", if include_uppercase { "✓" } else { "✗" });
        println!("2. Lowercase letters: {}", if include_lowercase { "✓" } else { "✗" });
        println!("3. Digits: {}", if include_digits { "✓" } else { "✗" });
        println!("4. Special characters: {}", if include_special { "✓" } else { "✗" });
        println!("5. Generate password");
    }
    
    if include_uppercase { char_sets.push(UPPERCASE.to_string()); }
    if include_lowercase { char_sets.push(LOWERCASE.to_string()); }
    if include_digits { char_sets.push(DIGITS.to_string()); }
    if include_special { char_sets.push(SPECIAL.to_string()); }
    
    if char_sets.is_empty() {
        println!("Warning: No character sets selected. Using all character sets.");
        char_sets.extend_from_slice(&[
            UPPERCASE.to_string(),
            LOWERCASE.to_string(),
            DIGITS.to_string(),
            SPECIAL.to_string()
        ]);
    }
    
    char_sets
}

fn generate_password(length: usize, char_sets: &[String]) -> String {
    let mut rng = rand::thread_rng();
    let all_chars: String = char_sets.concat();
    
    let mut password = String::with_capacity(length);
    
    for _ in 0..length {
        let idx = rng.gen_range(0..all_chars.len());
        password.push(all_chars.chars().nth(idx).unwrap());
    }
    
    password
}

fn assess_password_strength(password: &str) -> String {
    let length = password.len();
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    
    let mut score = 0;
    
    if length >= 12 { score += 2; }
    else if length >= 8 { score += 1; }
    
    if has_upper { score += 1; }
    if has_lower { score += 1; }
    if has_digit { score += 1; }
    if has_special { score += 2; }
    
    match score {
        0..=2 => "Weak",
        3..=4 => "Moderate",
        5..=6 => "Strong",
        _ => "Very Strong",
    }.to_string()
}