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
```