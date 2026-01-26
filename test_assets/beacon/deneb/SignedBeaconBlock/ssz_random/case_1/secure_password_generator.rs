use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordConfig {
    length: usize,
    include_uppercase: bool,
    include_lowercase: bool,
    include_digits: bool,
    include_special: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            include_uppercase: true,
            include_lowercase: true,
            include_digits: true,
            include_special: true,
        }
    }
}

impl PasswordConfig {
    fn validate(&self) -> Result<(), String> {
        if self.length < 8 {
            return Err("Password length must be at least 8 characters".to_string());
        }
        
        let mut char_set_count = 0;
        if self.include_uppercase { char_set_count += 1; }
        if self.include_lowercase { char_set_count += 1; }
        if self.include_digits { char_set_count += 1; }
        if self.include_special { char_set_count += 1; }
        
        if char_set_count < 2 {
            return Err("At least two character sets must be selected".to_string());
        }
        
        Ok(())
    }
    
    fn build_character_set(&self) -> String {
        let mut charset = String::new();
        if self.include_uppercase { charset.push_str(UPPERCASE); }
        if self.include_lowercase { charset.push_str(LOWERCASE); }
        if self.include_digits { charset.push_str(DIGITS); }
        if self.include_special { charset.push_str(SPECIAL); }
        charset
    }
}

fn generate_password(config: &PasswordConfig) -> String {
    let charset = config.build_character_set();
    let charset_bytes = charset.as_bytes();
    let mut rng = rand::thread_rng();
    
    (0..config.length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_bytes.len());
            charset_bytes[idx] as char
        })
        .collect()
}

fn get_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn parse_bool_input(input: &str) -> bool {
    match input.to_lowercase().as_str() {
        "y" | "yes" | "true" | "1" => true,
        _ => false,
    }
}

fn main() {
    println!("=== Secure Password Generator ===");
    
    let mut config = PasswordConfig::default();
    
    loop {
        let length_input = get_user_input(&format!("Password length (default: {}): ", DEFAULT_LENGTH));
        if !length_input.is_empty() {
            match length_input.parse::<usize>() {
                Ok(len) => config.length = len,
                Err(_) => println!("Invalid number, using default length"),
            }
        }
        
        let uppercase_input = get_user_input("Include uppercase letters? (Y/n): ");
        if !uppercase_input.is_empty() {
            config.include_uppercase = parse_bool_input(&uppercase_input);
        }
        
        let lowercase_input = get_user_input("Include lowercase letters? (Y/n): ");
        if !lowercase_input.is_empty() {
            config.include_lowercase = parse_bool_input(&lowercase_input);
        }
        
        let digits_input = get_user_input("Include digits? (Y/n): ");
        if !digits_input.is_empty() {
            config.include_digits = parse_bool_input(&digits_input);
        }
        
        let special_input = get_user_input("Include special characters? (Y/n): ");
        if !special_input.is_empty() {
            config.include_special = parse_bool_input(&special_input);
        }
        
        match config.validate() {
            Ok(_) => break,
            Err(err) => {
                println!("Configuration error: {}", err);
                println!("Please try again.\n");
                config = PasswordConfig::default();
            }
        }
    }
    
    let password = generate_password(&config);
    println!("\nGenerated password: {}", password);
    println!("Password length: {} characters", password.len());
    
    let mut char_types = Vec::new();
    if config.include_uppercase { char_types.push("uppercase letters"); }
    if config.include_lowercase { char_types.push("lowercase letters"); }
    if config.include_digits { char_types.push("digits"); }
    if config.include_special { char_types.push("special characters"); }
    
    println!("Character sets used: {}", char_types.join(", "));
}