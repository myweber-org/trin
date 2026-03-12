use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        
        let mut processed_count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }
            
            let columns: Vec<&str> = line.split(',').collect();
            
            if columns.len() > self.filter_column {
                if columns[self.filter_column] == self.filter_value {
                    let transformed_line = self.transform_line(&columns);
                    writeln!(output_file, "{}", transformed_line)?;
                    processed_count += 1;
                }
            }
        }
        
        Ok(processed_count)
    }
    
    fn transform_line(&self, columns: &[&str]) -> String {
        let mut transformed = Vec::new();
        
        for (i, column) in columns.iter().enumerate() {
            if i == 1 {
                transformed.push(column.to_uppercase());
            } else if i == 3 {
                transformed.push(format!("${}", column));
            } else {
                transformed.push(column.to_string());
            }
        }
        
        transformed.join(",")
    }
}

pub fn validate_csv_format(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return false;
    }
    
    let column_count = lines[0].split(',').count();
    
    for line in lines.iter().skip(1) {
        if line.split(',').count() != column_count {
            return false;
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    
    #[test]
    fn test_validate_csv_format() {
        let valid_csv = "id,name,age,salary\n1,john,30,50000\n2,jane,25,60000";
        assert!(validate_csv_format(valid_csv));
        
        let invalid_csv = "id,name,age,salary\n1,john,30\n2,jane,25,60000,extra";
        assert!(!validate_csv_format(invalid_csv));
    }
    
    #[test]
    fn test_csv_processor() -> Result<(), Box<dyn Error>> {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let test_data = "id,name,department,salary\n1,alice,engineering,75000\n2,bob,marketing,65000\n3,charlie,engineering,80000";
        
        let mut input_file = File::create(test_input)?;
        input_file.write_all(test_data.as_bytes())?;
        
        let processor = CsvProcessor::new(test_input, test_output, 2, "engineering");
        let processed = processor.process()?;
        
        assert_eq!(processed, 2);
        
        let mut output_file = File::open(test_output)?;
        let mut output_content = String::new();
        output_file.read_to_string(&mut output_content)?;
        
        assert!(output_content.contains("ALICE"));
        assert!(output_content.contains("$75000"));
        assert!(!output_content.contains("bob"));
        
        std::fs::remove_file(test_input)?;
        std::fs::remove_file(test_output)?;
        
        Ok(())
    }
}