
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

    pub fn process_string(&mut self, input: &str) -> Option<String> {
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

    pub fn process_batch(&mut self, inputs: &[&str]) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process_string(input))
            .collect()
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
    fn test_basic_cleaning() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process_string("  Hello  "), Some("hello".to_string()));
        assert_eq!(cleaner.process_string("HELLO"), None);
        assert_eq!(cleaner.process_string(""), None);
        assert_eq!(cleaner.process_string("   "), None);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let inputs = vec!["Apple", "apple", "Banana", "  banana  ", "Cherry"];
        
        let result = cleaner.process_batch(&inputs);
        assert_eq!(result.len(), 3);
        assert_eq!(cleaner.get_unique_count(), 3);
    }
}use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);
    let mut seen = HashSet::new();
    let mut unique_lines = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if seen.insert(line.clone()) {
            unique_lines.push(line);
        }
    }

    let mut output_file = File::create(output_path)?;
    for line in unique_lines {
        writeln!(output_file, "{}", line)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_remove_duplicates() {
        let input = "test_input.csv";
        let output = "test_output.csv";
        let test_data = "id,name\n1,alice\n2,bob\n1,alice\n3,charlie\n";

        std::fs::write(input, test_data).unwrap();
        remove_duplicates(input, output).unwrap();

        let mut content = String::new();
        File::open(output).unwrap().read_to_string(&mut content).unwrap();
        let expected = "id,name\n1,alice\n2,bob\n3,charlie\n";
        assert_eq!(content, expected);

        std::fs::remove_file(input).unwrap();
        std::fs::remove_file(output).unwrap();
    }
}