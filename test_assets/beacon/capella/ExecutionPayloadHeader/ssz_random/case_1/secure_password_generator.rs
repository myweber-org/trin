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
}use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
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

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_numbers: true,
            use_symbols: true,
        }
    }
}

fn generate_password(config: &PasswordConfig) -> String {
    let mut character_pool = String::new();
    
    if config.use_uppercase {
        character_pool.push_str(UPPERCASE);
    }
    if config.use_lowercase {
        character_pool.push_str(LOWERCASE);
    }
    if config.use_numbers {
        character_pool.push_str(NUMBERS);
    }
    if config.use_symbols {
        character_pool.push_str(SYMBOLS);
    }
    
    if character_pool.is_empty() {
        return String::from("Error: No character types selected");
    }
    
    let mut rng = rand::thread_rng();
    let password: String = (0..config.length)
        .map(|_| {
            let idx = rng.gen_range(0..character_pool.len());
            character_pool.chars().nth(idx).unwrap()
        })
        .collect();
    
    password
}

fn get_user_input() -> PasswordConfig {
    let mut config = PasswordConfig::default();
    
    println!("Password Generator");
    println!("==================");
    
    println!("Enter password length (default: {}): ", DEFAULT_LENGTH);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    if let Ok(length) = input.trim().parse::<usize>() {
        if length >= 4 && length <= 128 {
            config.length = length;
        } else {
            println!("Invalid length. Using default: {}", DEFAULT_LENGTH);
        }
    }
    
    println!("Include uppercase letters? (y/n, default: y): ");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    config.use_uppercase = !input.trim().eq_ignore_ascii_case("n");
    
    println!("Include lowercase letters? (y/n, default: y): ");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    config.use_lowercase = !input.trim().eq_ignore_ascii_case("n");
    
    println!("Include numbers? (y/n, default: y): ");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    config.use_numbers = !input.trim().eq_ignore_ascii_case("n");
    
    println!("Include symbols? (y/n, default: y): ");
    input.clear();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    config.use_symbols = !input.trim().eq_ignore_ascii_case("n");
    
    config
}

fn main() {
    let config = get_user_input();
    let password = generate_password(&config);
    
    println!("\nGenerated Password: {}", password);
    println!("Password Length: {}", password.len());
    
    let mut strength = "Weak";
    if password.len() >= 12 && config.use_uppercase && config.use_lowercase && config.use_numbers && config.use_symbols {
        strength = "Strong";
    } else if password.len() >= 8 {
        strength = "Medium";
    }
    
    println!("Estimated Strength: {}", strength);
}