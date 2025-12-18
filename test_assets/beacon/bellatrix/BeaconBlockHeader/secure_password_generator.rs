
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
        PasswordGenerator {
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

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than 0");
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
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password_chars = Vec::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        while password_chars.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx] as char;
            
            if !used_chars.contains(&ch) || password_chars.len() >= character_pool.len() {
                password_chars.push(ch);
                used_chars.insert(ch);
            }
        }

        Ok(password_chars.into_iter().collect())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .special(false);
        
        let password = generator.generate().unwrap();
        assert!(!password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(!password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)));
    }

    #[test]
    fn test_strength_validation() {
        assert!(matches!(
            PasswordGenerator::validate_strength("password"),
            StrengthLevel::Weak
        ));
        
        assert!(matches!(
            PasswordGenerator::validate_strength("Password123"),
            StrengthLevel::Medium
        ));
        
        assert!(matches!(
            PasswordGenerator::validate_strength("Str0ngP@ssw0rd!"),
            StrengthLevel::Strong
        ));
    }
}