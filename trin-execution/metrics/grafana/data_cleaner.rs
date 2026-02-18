use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataCleaner {
    file_path: String,
    delimiter: char,
}

impl DataCleaner {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        DataCleaner {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn validate_csv(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();
        let mut headers: Option<Vec<String>> = None;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let row_num = line_num + 1;

            if line.trim().is_empty() {
                errors.push(format!("Line {}: Empty row", row_num));
                continue;
            }

            let fields: Vec<&str> = line.split(self.delimiter).collect();

            if row_num == 1 {
                headers = Some(fields.iter().map(|s| s.trim().to_string()).collect());
                if headers.as_ref().unwrap().is_empty() {
                    errors.push("Header row is empty".to_string());
                }
                continue;
            }

            if let Some(ref header_row) = headers {
                if fields.len() != header_row.len() {
                    errors.push(format!(
                        "Line {}: Field count mismatch. Expected {}, found {}",
                        row_num,
                        header_row.len(),
                        fields.len()
                    ));
                }
            }

            for (col_num, field) in fields.iter().enumerate() {
                let trimmed = field.trim();
                if trimmed.is_empty() {
                    errors.push(format!(
                        "Line {}, Column {}: Empty field",
                        row_num,
                        col_num + 1
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(headers.unwrap_or_default())
        } else {
            Err(errors.join("\n").into())
        }
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let total_lines = reader.lines().count();
        Ok(if total_lines > 0 { total_lines - 1 } else { 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let cleaner = DataCleaner::new(temp_file.path().to_str().unwrap(), ',');
        let result = cleaner.validate_csv();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["name", "age", "city"]);
    }

    #[test]
    fn test_invalid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30").unwrap();
        writeln!(temp_file, "Alice,25,London,extra").unwrap();

        let cleaner = DataCleaner::new(temp_file.path().to_str().unwrap(), ',');
        let result = cleaner.validate_csv();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Field count mismatch"));
    }

    #[test]
    fn test_count_records() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let cleaner = DataCleaner::new(temp_file.path().to_str().unwrap(), ',');
        let count = cleaner.count_records().unwrap();
        assert_eq!(count, 2);
    }
}use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("Empty or invalid CSV file".into()),
    };
    
    let mut seen = HashSet::new();
    let mut unique_rows = Vec::new();
    
    for line_result in lines {
        let line = line_result?;
        if !seen.contains(&line) {
            seen.insert(line.clone());
            unique_rows.push(line);
        }
    }
    
    let mut output_file = File::create(Path::new(output_path))?;
    writeln!(output_file, "{}", header)?;
    
    for row in unique_rows {
        writeln!(output_file, "{}", row)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    
    #[test]
    fn test_remove_duplicates() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let test_data = "id,name,value\n1,Alice,100\n2,Bob,200\n1,Alice,100\n3,Charlie,300\n2,Bob,200";
        std::fs::write(test_input, test_data).unwrap();
        
        remove_duplicates(test_input, test_output).unwrap();
        
        let mut output_content = String::new();
        File::open(test_output)
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        let expected = "id,name,value\n1,Alice,100\n2,Bob,200\n3,Charlie,300\n";
        assert_eq!(output_content, expected);
        
        std::fs::remove_file(test_input).unwrap();
        std::fs::remove_file(test_output).unwrap();
    }
}