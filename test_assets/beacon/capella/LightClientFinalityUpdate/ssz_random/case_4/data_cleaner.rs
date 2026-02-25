use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct DataCleaner {
    input_path: String,
    output_path: String,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    pub fn clean_csv(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        
        let output_file = File::create(&self.output_path)?;
        let mut writer = std::io::BufWriter::new(output_file);

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 {
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;
                continue;
            }

            let cleaned_line = self.process_line(&line);
            if !cleaned_line.is_empty() {
                writer.write_all(cleaned_line.as_bytes())?;
                writer.write_all(b"\n")?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    fn process_line(&self, line: &str) -> String {
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() < 3 {
            return String::new();
        }

        let mut cleaned_parts = Vec::new();
        
        for part in parts {
            let trimmed = part.trim();
            if !trimmed.is_empty() && trimmed != "null" && trimmed != "NULL" {
                cleaned_parts.push(trimmed);
            } else {
                cleaned_parts.push("");
            }
        }

        cleaned_parts.join(",")
    }

    pub fn validate_file(&self) -> bool {
        Path::new(&self.input_path).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_data_cleaner() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let content = "id,name,value\n1,John,100\n2,,200\n3,Alice,null\n4,Bob,\n";
        fs::write(test_input, content).unwrap();

        let cleaner = DataCleaner::new(test_input, test_output);
        assert!(cleaner.validate_file());
        
        let result = cleaner.clean_csv();
        assert!(result.is_ok());

        let cleaned_content = fs::read_to_string(test_output).unwrap();
        assert!(cleaned_content.contains("1,John,100"));
        assert!(!cleaned_content.contains("null"));

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}