
use rand::Rng;
use std::collections::HashSet;

pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
        }
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn digits(mut self, enable: bool) -> Self {
        self.use_digits = enable;
        self
    }

    pub fn special(mut self, enable: bool) -> Self {
        self.use_special = enable;
        self
    }

    pub fn generate(&self) -> Result<String, String> {
        if self.length == 0 {
            return Err("Password length must be greater than 0".to_string());
        }

        let mut character_pool = Vec::new();
        
        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
        }
        
        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
        }
        
        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
        }
        
        if self.use_special {
            character_pool.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled".to_string());
        }

        let mut rng = rand::thread_rng();
        let mut password = String::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        while password.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx] as char;
            
            if used_chars.insert(ch) || password.len() < 3 {
                password.push(ch);
            }
        }

        Ok(password)
    }

    pub fn validate_strength(password: &str) -> StrengthLevel {
        let mut score = 0;
        
        if password.len() >= 12 {
            score += 2;
        } else if password.len() >= 8 {
            score += 1;
        }

        if password.chars().any(|c| c.is_ascii_uppercase()) {
            score += 1;
        }
        
        if password.chars().any(|c| c.is_ascii_lowercase()) {
            score += 1;
        }
        
        if password.chars().any(|c| c.is_ascii_digit()) {
            score += 1;
        }
        
        if password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            score += 1;
        }

        match score {
            0..=2 => StrengthLevel::Weak,
            3..=4 => StrengthLevel::Medium,
            _ => StrengthLevel::Strong,
        }
    }
}

pub enum StrengthLevel {
    Weak,
    Medium,
    Strong,
}

impl std::fmt::Display for StrengthLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrengthLevel::Weak => write!(f, "Weak"),
            StrengthLevel::Medium => write!(f, "Medium"),
            StrengthLevel::Strong => write!(f, "Strong"),
        }
    }
}