
fn clean_alphanumeric(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_alphanumeric() {
        assert_eq!(clean_alphanumeric("Hello, World! 123"), "HelloWorld123");
        assert_eq!(clean_alphanumeric("Rust_2024!"), "Rust2024");
        assert_eq!(clean_alphanumeric(""), "");
        assert_eq!(clean_alphanumeric("###"), "");
    }
}