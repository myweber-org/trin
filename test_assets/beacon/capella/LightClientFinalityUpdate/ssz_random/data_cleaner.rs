use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    
    println!("Enter data (press Ctrl+D on Unix or Ctrl+Z on Windows to finish):");
    
    for line in stdin.lock().lines() {
        match line {
            Ok(text) => buffer.push_str(&text),
            Err(e) => eprintln!("Error reading input: {}", e),
        }
    }
    
    let cleaned = clean_data(&buffer);
    println!("Cleaned data:");
    println!("{}", cleaned);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clean_data() {
        let input = "banana\napple\ncherry\nbanana\napple";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(clean_data(input), expected);
    }
    
    #[test]
    fn test_empty_input() {
        assert_eq!(clean_data(""), "");
    }
}