use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataCleaner {
    delimiter: char,
    has_header: bool,
}

impl DataCleaner {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataCleaner {
            delimiter,
            has_header,
        }
    }

    pub fn validate_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line?;
            
            if self.has_header && line_number == 1 {
                continue;
            }

            let fields: Vec<&str> = line.split(self.delimiter).collect();
            
            if fields.len() < 2 {
                errors.push(format!("Line {}: insufficient fields", line_number));
                continue;
            }

            for (i, field) in fields.iter().enumerate() {
                let trimmed = field.trim();
                
                if trimmed.is_empty() {
                    errors.push(format!("Line {}: empty field at column {}", line_number, i + 1));
                }
                
                if trimmed.contains('\n') || trimmed.contains('\r') {
                    errors.push(format!("Line {}: newline in field at column {}", line_number, i + 1));
                }
            }
        }

        Ok(errors)
    }

    pub fn clean_numeric_field(&self, value: &str) -> Option<f64> {
        let cleaned = value
            .trim()
            .replace(',', "")
            .replace('$', "")
            .replace(' ', "");
        
        cleaned.parse::<f64>().ok()
    }

    pub fn normalize_string(&self, input: &str) -> String {
        input
            .trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        writeln!(temp_file, "Jane,30,").unwrap();
        writeln!(temp_file, "Bob").unwrap();

        let cleaner = DataCleaner::new(',', true);
        let errors = cleaner.validate_csv(temp_file.path()).unwrap();
        
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("empty field"));
        assert!(errors[1].contains("insufficient fields"));
    }

    #[test]
    fn test_clean_numeric_field() {
        let cleaner = DataCleaner::new(',', false);
        
        assert_eq!(cleaner.clean_numeric_field("123.45"), Some(123.45));
        assert_eq!(cleaner.clean_numeric_field("$1,234.56"), Some(1234.56));
        assert_eq!(cleaner.clean_numeric_field("invalid"), None);
    }

    #[test]
    fn test_normalize_string() {
        let cleaner = DataCleaner::new(',', false);
        
        assert_eq!(
            cleaner.normalize_string("  Hello  World!  "),
            "hello world"
        );
        assert_eq!(
            cleaner.normalize_string("Data\tProcessing\nTest"),
            "data processing test"
        );
    }
}