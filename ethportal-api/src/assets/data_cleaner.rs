
use std::collections::HashMap;

pub struct DataCleaner {
    pub null_values: Vec<String>,
    pub string_normalizations: HashMap<String, String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            null_values: vec![
                "null".to_string(),
                "NULL".to_string(),
                "".to_string(),
                "N/A".to_string(),
                "n/a".to_string(),
            ],
            string_normalizations: HashMap::from([
                ("  ".to_string(), " ".to_string()),
                ("\t".to_string(), " ".to_string()),
                ("\n".to_string(), " ".to_string()),
            ]),
        }
    }

    pub fn clean_string(&self, input: &str) -> Option<String> {
        if self.null_values.contains(&input.to_string()) {
            return None;
        }

        let mut result = input.to_string();
        for (pattern, replacement) in &self.string_normalizations {
            result = result.replace(pattern, replacement);
        }

        result = result.trim().to_string();
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn clean_vector(&self, data: Vec<&str>) -> Vec<String> {
        data.iter()
            .filter_map(|&item| self.clean_string(item))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.clean_string("hello"), Some("hello".to_string()));
        assert_eq!(cleaner.clean_string(""), None);
        assert_eq!(cleaner.clean_string("null"), None);
        assert_eq!(cleaner.clean_string("  hello  "), Some("hello".to_string()));
        assert_eq!(cleaner.clean_string("hello\tworld"), Some("hello world".to_string()));
    }

    #[test]
    fn test_clean_vector() {
        let cleaner = DataCleaner::new();
        let data = vec!["hello", "", "null", "  test  "];
        let cleaned = cleaner.clean_vector(data);
        assert_eq!(cleaned, vec!["hello".to_string(), "test".to_string()]);
    }
}use std::collections::HashSet;

pub fn clean_and_sort_data<T: Ord + Clone>(data: &[T]) -> Vec<T> {
    let unique_items: HashSet<_> = data.iter().cloned().collect();
    let mut result: Vec<T> = unique_items.into_iter().collect();
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_sort_numbers() {
        let input = vec![5, 2, 8, 2, 5, 1, 9];
        let expected = vec![1, 2, 5, 8, 9];
        assert_eq!(clean_and_sort_data(&input), expected);
    }

    #[test]
    fn test_clean_and_sort_strings() {
        let input = vec!["banana", "apple", "cherry", "apple", "banana"];
        let expected = vec!["apple", "banana", "cherry"];
        assert_eq!(clean_and_sort_data(&input), expected);
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<i32> = vec![];
        let expected: Vec<i32> = vec![];
        assert_eq!(clean_and_sort_data(&input), expected);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct DataCleaner {
    input_path: String,
    output_path: String,
    delimiter: char,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            delimiter: ',',
        }
    }

    pub fn set_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn clean(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        let mut cleaned_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            let cleaned_line = self.process_line(&line, line_num + 1)?;
            
            if !cleaned_line.is_empty() {
                writeln!(output_file, "{}", cleaned_line)?;
                cleaned_count += 1;
            }
        }

        Ok(cleaned_count)
    }

    fn process_line(&self, line: &str, line_num: usize) -> Result<String, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() < 2 {
            eprintln!("Warning: Line {} has insufficient columns", line_num);
            return Ok(String::new());
        }

        let mut cleaned_parts = Vec::new();
        
        for (col_num, part) in parts.iter().enumerate() {
            let cleaned = part.trim();
            
            if cleaned.is_empty() {
                eprintln!("Warning: Line {}, Column {} is empty", line_num, col_num + 1);
                cleaned_parts.push("NULL");
            } else {
                cleaned_parts.push(cleaned);
            }
        }

        Ok(cleaned_parts.join(&self.delimiter.to_string()))
    }
}

pub fn validate_csv_path(path: &str) -> Result<(), Box<dyn Error>> {
    let path_obj = Path::new(path);
    
    if !path_obj.exists() {
        return Err(format!("File does not exist: {}", path).into());
    }
    
    if !path_obj.is_file() {
        return Err(format!("Path is not a file: {}", path).into());
    }
    
    if let Some(extension) = path_obj.extension() {
        if extension != "csv" {
            eprintln!("Warning: File extension is not .csv");
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
    fn test_data_cleaner_basic() {
        let input_content = "name,age,city\nJohn,25,NYC\nJane,30,LA\n";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let cleaner = DataCleaner::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        let result = cleaner.clean();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        assert_eq!(output_content, input_content);
    }

    #[test]
    fn test_clean_empty_lines() {
        let input_content = "name,age\n\nJohn,25\n\n\nJane,30\n";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let cleaner = DataCleaner::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        let cleaned_count = cleaner.clean().unwrap();
        assert_eq!(cleaned_count, 3);
        
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let expected = "name,age\nJohn,25\nJane,30\n";
        assert_eq!(output_content, expected);
    }
}