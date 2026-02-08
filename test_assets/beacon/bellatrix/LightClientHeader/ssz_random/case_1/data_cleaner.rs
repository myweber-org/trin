
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    
    sorted_lines.join("\n")
}

pub fn process_from_stdin() -> io::Result<()> {
    let stdin = io::stdin();
    let mut buffer = String::new();
    
    for line in stdin.lock().lines() {
        buffer.push_str(&line?);
        buffer.push('\n');
    }
    
    let cleaned = clean_data(&buffer);
    io::stdout().write_all(cleaned.as_bytes())?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clean_data() {
        let input = "cherry\napple\nbanana\napple\ncherry";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(clean_data(input), expected);
    }
    
    #[test]
    fn test_empty_input() {
        assert_eq!(clean_data(""), "");
    }
}use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

pub fn normalize_string(input: &str) -> String {
    input.trim().to_lowercase()
}

pub fn remove_empty_strings(strings: Vec<String>) -> Vec<String> {
    strings.into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let nums = vec![1, 2, 2, 3, 4, 4, 5];
        let deduped = deduplicate(nums);
        assert_eq!(deduped, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_normalize_string() {
        assert_eq!(normalize_string("  HELLO World  "), "hello world");
    }

    #[test]
    fn test_remove_empty_strings() {
        let strings = vec![
            "hello".to_string(),
            "".to_string(),
            "  ".to_string(),
            "world".to_string()
        ];
        let cleaned = remove_empty_strings(strings);
        assert_eq!(cleaned, vec!["hello".to_string(), "world".to_string()]);
    }
}