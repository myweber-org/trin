use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug)]
pub struct CsvConfig {
    pub input_path: String,
    pub output_path: String,
    pub filter_column: usize,
    pub filter_value: String,
}

pub fn process_csv(config: &CsvConfig) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(&config.input_path)?;
    let reader = BufReader::new(input_file);
    
    let output_file = File::create(&config.output_path)?;
    let mut writer = std::io::BufWriter::new(output_file);
    
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let fields: Vec<&str> = line.split(',').collect();
        
        if line_num == 0 {
            writeln!(writer, "{}", line)?;
            continue;
        }
        
        if fields.len() > config.filter_column {
            if fields[config.filter_column] == config.filter_value {
                writeln!(writer, "{}", line)?;
            }
        } else {
            eprintln!("Warning: Line {} has insufficient columns", line_num + 1);
        }
    }
    
    writer.flush()?;
    Ok(())
}

pub fn validate_config(config: &CsvConfig) -> Result<(), String> {
    if !Path::new(&config.input_path).exists() {
        return Err(format!("Input file not found: {}", config.input_path));
    }
    
    if config.filter_column == 0 {
        return Err("Filter column must be greater than 0".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_processing() {
        let input_content = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(input_content.as_bytes()).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let config = CsvConfig {
            input_path: input_file.path().to_str().unwrap().to_string(),
            output_path: output_file.path().to_str().unwrap().to_string(),
            filter_column: 2,
            filter_value: "active".to_string(),
        };
        
        assert!(validate_config(&config).is_ok());
        assert!(process_csv(&config).is_ok());
        
        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        let expected = "id,name,status\n1,alice,active\n3,charlie,active\n";
        assert_eq!(output_content, expected);
    }
    
    #[test]
    fn test_invalid_config() {
        let config = CsvConfig {
            input_path: "nonexistent.csv".to_string(),
            output_path: "output.csv".to_string(),
            filter_column: 0,
            filter_value: "test".to_string(),
        };
        
        assert!(validate_config(&config).is_err());
    }
}