use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().copied().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

pub fn process_stream() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    let mut buffer = String::new();

    for line in stdin.lock().lines() {
        let line = line?;
        buffer.push_str(&line);
        buffer.push('\n');
    }

    let cleaned = clean_data(&buffer);
    output.write_all(cleaned.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = "banana\napple\ncherry\napple\nbanana";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(clean_data(input), expected);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(clean_data(""), "");
    }
}
use std::collections::HashSet;

pub fn normalize_and_deduplicate(data: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for item in data {
        let normalized = item.trim().to_lowercase();
        if seen.insert(normalized.clone()) {
            result.push(normalized);
        }
    }

    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_and_deduplicate() {
        let input = vec![
            "  Apple ".to_string(),
            "apple".to_string(),
            "BANANA".to_string(),
            "banana ".to_string(),
            "Cherry".to_string(),
        ];
        let expected = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
        assert_eq!(normalize_and_deduplicate(input), expected);
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<String> = vec![];
        let result = normalize_and_deduplicate(input);
        assert!(result.is_empty());
    }
}