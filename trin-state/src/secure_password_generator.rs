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
}use rand::Rng;
use std::io;

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn main() {
    println!("Secure Password Generator");
    println!("==========================");

    let length = get_password_length();
    let char_sets = select_character_sets();
    
    if char_sets.is_empty() {
        println!("Error: At least one character set must be selected!");
        return;
    }

    let password = generate_password(length, &char_sets);
    println!("\nGenerated Password: {}", password);
    println!("Password Strength: {}", evaluate_strength(&password));
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (8-64):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        
        match input.trim().parse::<usize>() {
            Ok(length) if length >= 8 && length <= 64 => return length,
            Ok(_) => println!("Length must be between 8 and 64 characters"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn select_character_sets() -> Vec<String> {
    let mut char_sets = Vec::new();
    let mut rng = rand::thread_rng();

    println!("\nSelect character sets to include:");
    println!("1. Uppercase letters (A-Z)");
    println!("2. Lowercase letters (a-z)");
    println!("3. Digits (0-9)");
    println!("4. Symbols (!@#$% etc.)");
    println!("Enter numbers separated by spaces (e.g., '1 2 3 4'):");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    
    for num in input.split_whitespace() {
        match num {
            "1" => char_sets.push(UPPERCASE.to_string()),
            "2" => char_sets.push(LOWERCASE.to_string()),
            "3" => char_sets.push(DIGITS.to_string()),
            "4" => char_sets.push(SYMBOLS.to_string()),
            _ => println!("Ignoring invalid option: {}", num),
        }
    }

    // If no valid selections, use all character sets
    if char_sets.is_empty() {
        println!("No valid selections. Using all character sets by default.");
        char_sets = vec![
            UPPERCASE.to_string(),
            LOWERCASE.to_string(),
            DIGITS.to_string(),
            SYMBOLS.to_string(),
        ];
    }

    // Shuffle character sets for better randomness
    for i in 0..char_sets.len() {
        let j = rng.gen_range(0..char_sets.len());
        char_sets.swap(i, j);
    }

    char_sets
}

fn generate_password(length: usize, char_sets: &[String]) -> String {
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);
    
    // Ensure at least one character from each selected set
    for char_set in char_sets {
        if let Some(&ch) = char_set.as_bytes().choose(&mut rng) {
            password.push(ch as char);
        }
    }
    
    // Fill remaining length with random characters from all sets
    let all_chars: String = char_sets.concat();
    while password.len() < length {
        if let Some(&ch) = all_chars.as_bytes().choose(&mut rng) {
            password.push(ch as char);
        }
    }
    
    // Shuffle the password characters
    let mut chars: Vec<char> = password.chars().collect();
    chars.shuffle(&mut rng);
    chars.into_iter().collect()
}

fn evaluate_strength(password: &str) -> String {
    let length = password.len();
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_ascii_alphanumeric());
    
    let mut score = 0;
    if length >= 12 { score += 2; }
    else if length >= 8 { score += 1; }
    
    if has_upper { score += 1; }
    if has_lower { score += 1; }
    if has_digit { score += 1; }
    if has_symbol { score += 2; }
    
    match score {
        0..=2 => "Weak".to_string(),
        3..=4 => "Moderate".to_string(),
        5..=6 => "Strong".to_string(),
        _ => "Very Strong".to_string(),
    }
}