use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;

    for result in reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| field.trim().to_string())
            .collect();
        writer.write_record(&cleaned_record)?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name , age, city  ").unwrap();
        writeln!(input_file, "Alice, 25 ,  New York").unwrap();
        writeln!(input_file, "Bob  ,30,London ").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        clean_csv(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap()).unwrap();

        let mut reader = Reader::from_path(output_file.path()).unwrap();
        let records: Vec<_> = reader.records().collect::<Result<_, _>>().unwrap();

        assert_eq!(records[0][0], "name");
        assert_eq!(records[0][1], "age");
        assert_eq!(records[0][2], "city");
        assert_eq!(records[1][0], "Alice");
        assert_eq!(records[1][1], "25");
        assert_eq!(records[1][2], "New York");
        assert_eq!(records[2][0], "Bob");
        assert_eq!(records[2][1], "30");
        assert_eq!(records[2][2], "London");
    }
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
        
        assert_eq!(cleaner.process("hello"), Some("hello".to_string()));
        assert_eq!(cleaner.process("HELLO"), None);
        assert_eq!(cleaner.process("  hello  "), None);
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
        
        cleaner.process("test");
        assert_eq!(cleaner.get_unique_count(), 1);
    }
}