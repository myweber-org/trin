use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataCleaner {
    pub delimiter: char,
    pub skip_header: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            delimiter: ',',
            skip_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn clean_csv(&self, file_path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut cleaned_data = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if self.skip_header && index == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                cleaned_data.push(fields);
            }
        }

        Ok(cleaned_data)
    }

    pub fn convert_column_to_numeric(&self, data: &[Vec<String>], column_index: usize) -> Vec<Option<f64>> {
        data.iter()
            .map(|row| {
                if column_index < row.len() {
                    row[column_index].parse::<f64>().ok()
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn filter_valid_rows(&self, data: Vec<Vec<String>>) -> Vec<Vec<String>> {
        data.into_iter()
            .filter(|row| !row.is_empty() && row.iter().any(|field| !field.is_empty()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();

        let cleaner = DataCleaner::new();
        let result = cleaner.clean_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "25", "New York"]);
    }

    #[test]
    fn test_convert_column_to_numeric() {
        let data = vec![
            vec!["10".to_string(), "20".to_string()],
            vec!["invalid".to_string(), "30".to_string()],
            vec!["40".to_string()],
        ];
        
        let cleaner = DataCleaner::new();
        let numeric = cleaner.convert_column_to_numeric(&data, 0);
        
        assert_eq!(numeric, vec![Some(10.0), None, Some(40.0)]);
    }
}