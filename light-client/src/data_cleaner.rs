rust
pub fn normalize_string(input: &str) -> String {
    input.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_string() {
        assert_eq!(normalize_string("  Hello World  "), "hello world");
        assert_eq!(normalize_string("RUST Programming"), "rust programming");
        assert_eq!(normalize_string("ALLCAPS"), "allcaps");
        assert_eq!(normalize_string("  mixed CASE  "), "mixed case");
    }
}
```use regex::Regex;

pub fn clean_string(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();
    re.replace_all(input, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() {
        assert_eq!(clean_string("Hello, World! 123"), "HelloWorld123");
        assert_eq!(clean_string("Test@#$%^&*()String"), "TestString");
        assert_eq!(clean_string(""), "");
        assert_eq!(clean_string("123_456"), "123456");
    }
}