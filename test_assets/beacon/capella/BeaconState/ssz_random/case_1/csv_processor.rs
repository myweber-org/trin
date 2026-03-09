
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvFilter {
    input_path: String,
    output_path: String,
    selected_columns: Vec<usize>,
    delimiter: char,
}

impl CsvFilter {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvFilter {
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
                writeln!(output_file)?;
                continue;
            }
            
            let fields: Vec<&str> = line.split(self.delimiter).collect();
            
            if self.selected_columns.is_empty() {
                writeln!(output_file, "{}", line)?;
            } else {
                let selected_fields: Vec<String> = self.selected_columns
                    .iter()
                    .filter_map(|&idx| fields.get(idx).map(|s| s.to_string()))
                    .collect();
                
                writeln!(output_file, "{}", selected_fields.join(&self.delimiter.to_string()))?;
            }
            
            if line_num % 1000 == 0 && line_num > 0 {
                eprintln!("Processed {} lines...", line_num);
            }
        }
        
        Ok(())
    }
}

pub fn filter_csv(
    input_path: &str,
    output_path: &str,
    columns: Option<&[usize]>,
    delimiter: Option<char>,
) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvFilter::new(input_path, output_path);
    
    if let Some(cols) = columns {
        processor.select_columns(cols);
    }
    
    if let Some(delim) = delimiter {
        processor.set_delimiter(delim);
    }
    
    processor.process()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_filtering() -> Result<(), Box<dyn Error>> {
        let input_content = "name,age,city\nAlice,30,London\nBob,25,Paris\n";
        let input_file = NamedTempFile::new()?;
        fs::write(&input_file, input_content)?;
        
        let output_file = NamedTempFile::new()?;
        
        filter_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some(&[0, 2]),
            None,
        )?;
        
        let output_content = fs::read_to_string(output_file.path())?;
        assert_eq!(output_content, "name,city\nAlice,London\nBob,Paris\n");
        
        Ok(())
    }

    #[test]
    fn test_custom_delimiter() -> Result<(), Box<dyn Error>> {
        let input_content = "name|age|city\nAlice|30|London\n";
        let input_file = NamedTempFile::new()?;
        fs::write(&input_file, input_content)?;
        
        let output_file = NamedTempFile::new()?;
        
        let mut processor = CsvFilter::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        );
        
        processor.set_delimiter('|').select_columns(&[0, 1]).process()?;
        
        let output_content = fs::read_to_string(output_file.path())?;
        assert_eq!(output_content, "name|age\nAlice|30\n");
        
        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    valid: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        let valid = value >= 0.0 && value <= 1000.0;
        Record {
            id,
            name,
            value,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line_num == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            continue;
        }

        let id = match parts[0].parse::<u32>() {
            Ok(val) => val,
            Err(_) => continue,
        };

        let name = parts[1].to_string();
        let value = match parts[2].parse::<f64>() {
            Ok(val) => val,
            Err(_) => continue,
        };

        records.push(Record::new(id, name, value));
    }

    Ok(records)
}

pub fn filter_valid_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.is_valid()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "test".to_string(), 500.0);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "test".to_string(), 1500.0);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,name,value")?;
        writeln!(temp_file, "1,alpha,42.5")?;
        writeln!(temp_file, "2,beta,999.9")?;
        writeln!(temp_file, "3,gamma,1500.0")?;

        let records = process_csv_file(temp_file.path())?;
        assert_eq!(records.len(), 3);

        let valid_records = filter_valid_records(&records);
        assert_eq!(valid_records.len(), 2);

        Ok(())
    }
}