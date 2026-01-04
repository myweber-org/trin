use rand::Rng;
use std::io;

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordConfig {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_numbers: bool,
    use_symbols: bool,
}

impl PasswordConfig {
    fn new() -> Self {
        PasswordConfig {
            length: 16,
            use_uppercase: true,
            use_lowercase: true,
            use_numbers: true,
            use_symbols: true,
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.length < 8 {
            return Err("Password length must be at least 8 characters".to_string());
        }
        if !self.use_uppercase && !self.use_lowercase && !self.use_numbers && !self.use_symbols {
            return Err("At least one character set must be selected".to_string());
        }
        Ok(())
    }

    fn get_character_set(&self) -> String {
        let mut charset = String::new();
        if self.use_uppercase {
            charset.push_str(UPPERCASE);
        }
        if self.use_lowercase {
            charset.push_str(LOWERCASE);
        }
        if self.use_numbers {
            charset.push_str(NUMBERS);
        }
        if self.use_symbols {
            charset.push_str(SYMBOLS);
        }
        charset
    }
}

fn generate_password(config: &PasswordConfig) -> String {
    let charset = config.get_character_set();
    let charset_bytes = charset.as_bytes();
    let mut rng = rand::thread_rng();
    
    (0..config.length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_bytes.len());
            charset_bytes[idx] as char
        })
        .collect()
}

fn get_user_input() -> PasswordConfig {
    let mut config = PasswordConfig::new();
    
    println!("Password Generator Configuration");
    println!("--------------------------------");
    
    loop {
        println!("Enter password length (default: {}): ", config.length);
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if !input.is_empty() {
            match input.parse::<usize>() {
                Ok(length) => config.length = length,
                Err(_) => println!("Invalid number, using default length"),
            }
        }
        break;
    }
    
    println!("Include uppercase letters? (Y/n): ");
    config.use_uppercase = read_yes_no(true);
    
    println!("Include lowercase letters? (Y/n): ");
    config.use_lowercase = read_yes_no(true);
    
    println!("Include numbers? (Y/n): ");
    config.use_numbers = read_yes_no(true);
    
    println!("Include symbols? (Y/n): ");
    config.use_symbols = read_yes_no(true);
    
    config
}

fn read_yes_no(default: bool) -> bool {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();
    
    match input.as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        "" => default,
        _ => {
            println!("Invalid input, using default");
            default
        }
    }
}

fn main() {
    let config = get_user_input();
    
    match config.validate() {
        Ok(_) => {
            let password = generate_password(&config);
            println!("\nGenerated Password: {}", password);
            println!("Password Length: {}", password.len());
            
            let mut charset_info = Vec::new();
            if config.use_uppercase { charset_info.push("Uppercase"); }
            if config.use_lowercase { charset_info.push("Lowercase"); }
            if config.use_numbers { charset_info.push("Numbers"); }
            if config.use_symbols { charset_info.push("Symbols"); }
            
            println!("Character sets used: {}", charset_info.join(", "));
        }
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    }
}