
use regex::Regex;

pub fn clean_string(input: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    let trimmed = input.trim();
    let normalized = re.replace_all(trimmed, " ");
    normalized.to_string()
}

pub fn clean_string_lowercase(input: &str) -> String {
    clean_string(input).to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() {
        assert_eq!(clean_string("  hello   world  "), "hello world");
        assert_eq!(clean_string("data\n\twith\tspaces"), "data with spaces");
    }

    #[test]
    fn test_clean_string_lowercase() {
        assert_eq!(clean_string_lowercase("  HELLO   World  "), "hello world");
    }
}