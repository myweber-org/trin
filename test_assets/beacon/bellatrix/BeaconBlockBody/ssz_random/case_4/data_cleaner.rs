use std::error::Error;
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

    pub fn set_delimiter(&mut self, delimiter: char) -> &mut Self {
        self.delimiter = delimiter;
        self
    }

    pub fn clean(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        let mut cleaned_count = 0;

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
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
        let mut fields: Vec<String> = line
            .split(self.delimiter)
            .map(|field| field.trim().to_string())
            .collect();

        if fields.is_empty() {
            return Ok(String::new());
        }

        for field in fields.iter_mut() {
            if field.is_empty() {
                *field = "NULL".to_string();
            } else if field.parse::<f64>().is_ok() {
                let value: f64 = field.parse()?;
                if value.is_nan() || value.is_infinite() {
                    *field = "0.0".to_string();
                }
            }
        }

        Ok(fields.join(&self.delimiter.to_string()))
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
            return Err(format!("File must have .csv extension: {}", path).into());
        }
    } else {
        return Err(format!("File has no extension: {}", path).into());
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
        let input_content = "name,age,city\nJohn,25,NYC\nJane,,London\nBob,invalid,Berlin";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(&input_file, input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let cleaner = DataCleaner::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        let result = cleaner.clean();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("NULL"));
    }

    #[test]
    fn test_validate_csv_path() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap();
        
        let result = validate_csv_path(temp_path);
        assert!(result.is_err());
        
        let csv_file = NamedTempFile::new().unwrap();
        let csv_path = csv_file.path().with_extension("csv");
        fs::write(&csv_path, "").unwrap();
        
        let result = validate_csv_path(csv_path.to_str().unwrap());
        assert!(result.is_ok());
    }
}
use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, input: &str) -> Option<String> {
        if self.dedupe_set.insert(input.to_string()) {
            Some(input.to_string())
        } else {
            None
        }
    }

    pub fn validate_email(&self, email: &str) -> Result<(), Box<dyn Error>> {
        if email.contains('@') && email.contains('.') {
            Ok(())
        } else {
            Err("Invalid email format".into())
        }
    }

    pub fn normalize_whitespace(input: &str) -> String {
        input
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn remove_special_chars(input: &str, allowed: &[char]) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || allowed.contains(c))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        assert_eq!(cleaner.deduplicate("test"), Some("test".to_string()));
        assert_eq!(cleaner.deduplicate("test"), None);
    }

    #[test]
    fn test_validate_email() {
        let cleaner = DataCleaner::new();
        assert!(cleaner.validate_email("user@example.com").is_ok());
        assert!(cleaner.validate_email("invalid").is_err());
    }

    #[test]
    fn test_normalize_whitespace() {
        let input = "  multiple   spaces   here  ";
        let expected = "multiple spaces here";
        assert_eq!(DataCleaner::normalize_whitespace(input), expected);
    }

    #[test]
    fn test_remove_special_chars() {
        let input = "Hello, @World! #2024";
        let allowed = ['@', '!'];
        let result = DataCleaner::remove_special_chars(input, &allowed);
        assert_eq!(result, "Hello@World!2024");
    }
}