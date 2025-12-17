use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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
        let mut output_file = File::create(&self.output_path)?;

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let cleaned_line = self.process_line(&line);
            if !cleaned_line.is_empty() {
                writeln!(output_file, "{}", cleaned_line)?;
            }
        }

        Ok(())
    }

    fn process_line(&self, line: &str) -> String {
        let parts: Vec<&str> = line.split(',').collect();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_data_cleaner() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let content = "id,name,value\n1,John,100\n2,,200\n3,Alice,null\n4,NULL,300";
        fs::write(test_input, content).unwrap();

        let cleaner = DataCleaner::new(test_input, test_output);
        let result = cleaner.clean_csv();
        
        assert!(result.is_ok());
        
        let cleaned_content = fs::read_to_string(test_output).unwrap();
        let expected = "id,name,value\n1,John,100\n2,,200\n3,Alice,\n4,,300\n";
        assert_eq!(cleaned_content, expected);

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}