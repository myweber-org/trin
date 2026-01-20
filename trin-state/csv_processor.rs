use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str, filter_column: usize, filter_value: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_number == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if parts.len() > self.filter_column && parts[self.filter_column] == self.filter_value {
                let transformed_line = parts
                    .iter()
                    .map(|&part| part.trim().to_uppercase())
                    .collect::<Vec<String>>()
                    .join(",");
                writeln!(output_file, "{}", transformed_line)?;
            }
        }

        Ok(())
    }

    pub fn count_matching_rows(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut count = 0;

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_number == 0 {
                continue;
            }

            if parts.len() > self.filter_column && parts[self.filter_column] == self.filter_value {
                count += 1;
            }
        }

        Ok(count)
    }
}

pub fn validate_csv_format(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return false;
    }

    let column_count = lines[0].split(',').count();
    lines.iter().all(|line| line.split(',').count() == column_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_input = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";

        fs::write(input_path, test_input).unwrap();

        let processor = CsvProcessor::new(input_path, output_path, 2, "active");
        let result = processor.process();
        assert!(result.is_ok());

        let count = processor.count_matching_rows().unwrap();
        assert_eq!(count, 2);

        let output_content = fs::read_to_string(output_path).unwrap();
        assert!(output_content.contains("ALICE"));
        assert!(!output_content.contains("BOB"));

        fs::remove_file(input_path).unwrap();
        fs::remove_file(output_path).unwrap();
    }

    #[test]
    fn test_csv_validation() {
        let valid_csv = "a,b,c\n1,2,3\n4,5,6";
        let invalid_csv = "a,b,c\n1,2\n3,4,5,6";

        assert!(validate_csv_format(valid_csv));
        assert!(!validate_csv_format(invalid_csv));
    }
}