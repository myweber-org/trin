use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: Option<usize>,
    filter_value: Option<String>,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column: None,
            filter_value: None,
        }
    }

    pub fn set_filter(&mut self, column: usize, value: &str) -> &mut Self {
        self.filter_column = Some(column);
        self.filter_value = Some(value.to_string());
        self
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let columns: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let should_include = match (self.filter_column, &self.filter_value) {
                (Some(col), Some(val)) if col < columns.len() => columns[col] == val,
                _ => true,
            };

            if should_include {
                let transformed_line = self.transform_line(&columns);
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    fn transform_line(&self, columns: &[&str]) -> String {
        columns
            .iter()
            .map(|col| col.trim().to_uppercase())
            .collect::<Vec<String>>()
            .join(",")
    }
}

pub fn validate_csv_path(path: &str) -> Result<(), String> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    if path_obj.extension().and_then(|ext| ext.to_str()) != Some("csv") {
        return Err("File must have .csv extension".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        let content = "id,name,value\n1,test,100\n2,demo,200\n3,test,300";
        
        fs::write(test_input, content).unwrap();
        
        let mut processor = CsvProcessor::new(test_input, test_output);
        processor.set_filter(1, "test");
        
        let result = processor.process();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let output_content = fs::read_to_string(test_output).unwrap();
        assert!(output_content.contains("TEST"));
        assert!(!output_content.contains("DEMO"));
        
        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}