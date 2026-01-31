
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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
        
        let output_file = File::create(&self.output_path)?;
        let mut writer = std::io::BufWriter::new(output_file);
        
        let mut processed_count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                writeln!(writer, "{}", line)?;
                continue;
            }
            
            let columns: Vec<&str> = line.split(',').collect();
            
            if columns.len() > self.filter_column {
                if columns[self.filter_column] == self.filter_value {
                    writeln!(writer, "{}", line)?;
                    processed_count += 1;
                }
            }
        }
        
        Ok(processed_count)
    }
    
    pub fn validate_paths(&self) -> Result<(), Box<dyn Error>> {
        if !Path::new(&self.input_path).exists() {
            return Err(format!("Input file not found: {}", self.input_path).into());
        }
        
        let output_dir = Path::new(&self.output_path).parent();
        if let Some(dir) = output_dir {
            if !dir.exists() {
                return Err(format!("Output directory does not exist: {:?}", dir).into());
            }
        }
        
        Ok(())
    }
}

pub fn process_csv_file(
    input_path: &str,
    output_path: &str,
    filter_column: usize,
    filter_value: &str,
) -> Result<usize, Box<dyn Error>> {
    let processor = CsvProcessor::new(input_path, output_path, filter_column, filter_value);
    processor.validate_paths()?;
    processor.process()
}