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
    let mut buffer = String::new();
    
    println!("Enter data (press Ctrl+D on Unix or Ctrl+Z on Windows to finish):");
    for line in stdin.lock().lines() {
        buffer.push_str(&line?);
        buffer.push('\n');
    }
    
    let cleaned = clean_data(&buffer);
    
    let mut output_file = std::fs::File::create("cleaned_output.txt")?;
    output_file.write_all(cleaned.as_bytes())?;
    
    println!("Data cleaned and saved to cleaned_output.txt");
    Ok(())
}use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, text: &str) -> String {
        text.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<String>) -> Vec<String> {
        data.into_iter()
            .filter(|item| self.deduplicate(item))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World!  "), "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("Hello"));
        assert!(!cleaner.deduplicate("hello"));
        assert!(cleaner.deduplicate("World"));
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_dataset_cleaning() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "Banana".to_string(),
            "banana ".to_string(),
        ];
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaned[0], "Apple");
        assert_eq!(cleaned[1], "Banana");
    }
}