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
    pub fn new(input: &str, output: &str, column: usize, value: &str) -> Self {
        CsvProcessor {
            input_path: input.to_string(),
            output_path: output.to_string(),
            filter_column,
            filter_value: value.to_string(),
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
            
            if columns.get(self.filter_column)
                .map(|val| val.trim() == self.filter_value)
                .unwrap_or(false)
            {
                let transformed_line = columns
                    .iter()
                    .map(|col| col.trim().to_uppercase())
                    .collect::<Vec<String>>()
                    .join(",");
                
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }
}

pub fn validate_csv_header(file_path: &str) -> Result<bool, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    
    if let Some(first_line) = reader.lines().next() {
        let header = first_line?;
        return Ok(!header.is_empty() && header.contains(','));
    }
    
    Ok(false)
}