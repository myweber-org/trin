use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("Empty or invalid CSV file".into()),
    };
    
    let mut seen = HashSet::new();
    let mut unique_lines = Vec::new();
    
    for line_result in lines {
        let line = line_result?;
        if !seen.contains(&line) {
            seen.insert(line.clone());
            unique_lines.push(line);
        }
    }
    
    let mut output_file = File::create(output_path)?;
    writeln!(output_file, "{}", header)?;
    
    for line in unique_lines {
        writeln!(output_file, "{}", line)?;
    }
    
    Ok(())
}use regex::Regex;

pub fn sanitize_input(input: &str) -> String {
    let trimmed = input.trim();
    
    let re = Regex::new(r"\s+").unwrap();
    let normalized_whitespace = re.replace_all(trimmed, " ");
    
    normalized_whitespace.to_string()
}

pub fn remove_special_chars(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9\s]").unwrap();
    re.replace_all(input, "").to_string()
}

pub fn normalize_case(input: &str) -> String {
    input.to_lowercase()
}

pub fn clean_data(input: &str) -> String {
    let sanitized = sanitize_input(input);
    let cleaned = remove_special_chars(&sanitized);
    normalize_case(&cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_input() {
        assert_eq!(sanitize_input("  hello   world  "), "hello world");
        assert_eq!(sanitize_input("data\twith\ttabs"), "data with tabs");
    }

    #[test]
    fn test_remove_special_chars() {
        assert_eq!(remove_special_chars("hello@world!"), "hello world");
        assert_eq!(remove_special_chars("test#123"), "test123");
    }

    #[test]
    fn test_clean_data() {
        assert_eq!(clean_data("  Hello@World!  "), "hello world");
        assert_eq!(clean_data("TEST\t#123"), "test 123");
    }
}