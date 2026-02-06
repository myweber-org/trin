use std::collections::HashSet;
use std::io::{self, BufRead, Write};

fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut input = String::new();
    
    println!("Enter data (press Ctrl+D on Unix or Ctrl+Z on Windows to finish):");
    for line in stdin.lock().lines() {
        input.push_str(&line?);
        input.push('\n');
    }
    
    let cleaned = clean_data(&input);
    
    let mut output_file = std::fs::File::create("cleaned_output.txt")?;
    output_file.write_all(cleaned.as_bytes())?;
    
    println!("Data cleaned and saved to cleaned_output.txt");
    Ok(())
}
use std::collections::HashSet;

pub struct DataCleaner {
    unique_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_items: HashSet::new(),
        }
    }

    pub fn process(&mut self, input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        
        if normalized.is_empty() {
            return None;
        }

        if self.unique_items.contains(&normalized) {
            return None;
        }

        self.unique_items.insert(normalized.clone());
        Some(normalized)
    }

    pub fn get_unique_count(&self) -> usize {
        self.unique_items.len()
    }

    pub fn clear(&mut self) {
        self.unique_items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_removal() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process("Hello"), Some("hello".to_string()));
        assert_eq!(cleaner.process("  HELLO  "), None);
        assert_eq!(cleaner.process("world"), Some("world".to_string()));
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_empty_input() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process(""), None);
        assert_eq!(cleaner.process("   "), None);
        assert_eq!(cleaner.get_unique_count(), 0);
    }

    #[test]
    fn test_clear_function() {
        let mut cleaner = DataCleaner::new();
        
        cleaner.process("test");
        assert_eq!(cleaner.get_unique_count(), 1);
        
        cleaner.clear();
        assert_eq!(cleaner.get_unique_count(), 0);
    }
}