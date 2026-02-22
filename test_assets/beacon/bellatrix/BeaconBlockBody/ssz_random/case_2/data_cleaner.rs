use std::collections::HashSet;

pub struct DataCleaner;

impl DataCleaner {
    pub fn deduplicate_strings(strings: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        strings
            .into_iter()
            .filter(|s| seen.insert(s.clone()))
            .collect()
    }

    pub fn normalize_whitespace(input: &str) -> String {
        input
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn remove_empty_lines(text: &str) -> String {
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<&str>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_strings() {
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
            "banana".to_string(),
        ];
        let result = DataCleaner::deduplicate_strings(input);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
        assert!(result.contains(&"cherry".to_string()));
    }

    #[test]
    fn test_normalize_whitespace() {
        let input = "  hello    world  \t\n  test  ";
        let result = DataCleaner::normalize_whitespace(input);
        assert_eq!(result, "hello world test");
    }

    #[test]
    fn test_remove_empty_lines() {
        let input = "line1\n\nline2\n  \nline3\n\t\nline4";
        let result = DataCleaner::remove_empty_lines(input);
        assert_eq!(result, "line1\nline2\nline3\nline4");
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    deduplication_cache: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            deduplication_cache: HashSet::new(),
        }
    }

    pub fn normalize_string(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn is_duplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_string(item);
        !self.deduplication_cache.insert(normalized)
    }

    pub fn clean_dataset(&mut self, dataset: Vec<String>) -> Vec<String> {
        let mut cleaned = Vec::new();
        
        for item in dataset {
            if !self.is_duplicate(&item) {
                cleaned.push(item);
            }
        }
        
        cleaned
    }

    pub fn reset_cache(&mut self) {
        self.deduplication_cache.clear();
    }

    pub fn get_unique_count(&self) -> usize {
        self.deduplication_cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_string("  HELLO World  "), "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let dataset = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "Banana".to_string(),
            "  apple  ".to_string(),
        ];
        
        let cleaned = cleaner.clean_dataset(dataset);
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }
}
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(input_path);
    let output_path = Path::new(output_path);
    
    if !input_path.exists() {
        return Err(format!("Input file '{}' does not exist", input_path.display()).into());
    }
    
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut lines = reader.lines();
    
    let mut seen = HashSet::new();
    let mut output_file = File::create(output_path)?;
    
    if let Some(header) = lines.next() {
        let header_line = header?;
        writeln!(output_file, "{}", header_line)?;
    }
    
    for line_result in lines {
        let line = line_result?;
        if !seen.contains(&line) {
            writeln!(output_file, "{}", line)?;
            seen.insert(line);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_remove_duplicates() {
        let input_content = "id,name,value\n1,Alice,100\n2,Bob,200\n1,Alice,100\n3,Charlie,300\n2,Bob,200";
        let expected_output = "id,name,value\n1,Alice,100\n2,Bob,200\n3,Charlie,300";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), input_content).unwrap();
        
        remove_duplicates(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        let actual_output = fs::read_to_string(output_file.path()).unwrap();
        assert_eq!(actual_output.trim(), expected_output);
    }
    
    #[test]
    fn test_file_not_found() {
        let result = remove_duplicates("nonexistent.csv", "output.csv");
        assert!(result.is_err());
    }
}