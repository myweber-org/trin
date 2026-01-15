use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    selected_columns: Vec<usize>,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            selected_columns: Vec::new(),
            delimiter: ',',
        }
    }

    pub fn select_columns(&mut self, columns: &[usize]) -> &mut Self {
        self.selected_columns = columns.to_vec();
        self
    }

    pub fn set_delimiter(&mut self, delimiter: char) -> &mut Self {
        self.delimiter = delimiter;
        self
    }

    pub fn process(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<&str> = line.split(self.delimiter).collect();
            
            if self.selected_columns.is_empty() {
                writeln!(output_file, "{}", line)?;
            } else {
                let selected_fields: Vec<&str> = self.selected_columns
                    .iter()
                    .filter_map(|&idx| fields.get(idx).copied())
                    .collect();
                
                if !selected_fields.is_empty() {
                    writeln!(output_file, "{}", selected_fields.join(&self.delimiter.to_string()))?;
                } else if line_num == 0 {
                    return Err("No valid columns selected".into());
                }
            }
        }

        Ok(())
    }
}

pub fn filter_csv_data(input: &str, output: &str, columns: Option<&[usize]>) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new(input, output);
    
    if let Some(cols) = columns {
        processor.select_columns(cols);
    }
    
    processor.process()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_basic_filtering() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let test_data = "id,name,age,city\n1,Alice,30,London\n2,Bob,25,Paris\n3,Charlie,35,Berlin";
        fs::write(test_input, test_data).unwrap();

        let result = filter_csv_data(test_input, test_output, Some(&[0, 1]));
        assert!(result.is_ok());

        let output_content = fs::read_to_string(test_output).unwrap();
        assert!(output_content.contains("id,name"));
        assert!(output_content.contains("1,Alice"));
        assert!(!output_content.contains("30"));

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }

    #[test]
    fn test_custom_delimiter() {
        let test_input = "test_delim.csv";
        let test_output = "test_delim_out.csv";
        
        let test_data = "id|name|age\n1|Alice|30\n2|Bob|25";
        fs::write(test_input, test_data).unwrap();

        let mut processor = CsvProcessor::new(test_input, test_output);
        processor.set_delimiter('|').select_columns(&[1, 2]);
        
        let result = processor.process();
        assert!(result.is_ok());

        let output_content = fs::read_to_string(test_output).unwrap();
        assert!(output_content.contains("name|age"));
        assert!(output_content.contains("Alice|30"));

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}