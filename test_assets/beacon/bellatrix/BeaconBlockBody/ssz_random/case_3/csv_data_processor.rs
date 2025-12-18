use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn transform(&mut self) {
        self.name = self.name.to_uppercase();
        self.category = self.category.to_lowercase();
        self.value = (self.value * 100.0).round() / 100.0;
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = parts[0].parse::<u32>().unwrap_or(0);
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>().unwrap_or(0.0);
            let category = parts[3].to_string();
            
            let record = CsvRecord::new(id, name, value, category);
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn process_records(&mut self) -> Vec<String> {
        let mut errors = Vec::new();
        let mut valid_records = Vec::new();
        
        for (index, record) in self.records.iter_mut().enumerate() {
            match record.validate() {
                Ok(_) => {
                    record.transform();
                    valid_records.push(record.clone());
                }
                Err(err) => {
                    errors.push(format!("Record {} error: {}", index + 1, err));
                }
            }
        }
        
        if !errors.is_empty() {
            eprintln!("Processing errors: {:?}", errors);
        }
        
        valid_records
            .iter()
            .map(|r| format!("{},{},{:.2},{}", r.id, r.name, r.value, r.category))
            .collect()
    }

    pub fn save_to_file(&self, file_path: &str, data: &[String]) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(file_path)?;
        writeln!(file, "ID,NAME,VALUE,CATEGORY")?;
        
        for line in data {
            writeln!(file, "{}", line)?;
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let average = sum / count;
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        (average, min, max)
    }
}

pub fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    processor.load_from_file(input_path)?;
    
    let processed_data = processor.process_records();
    processor.save_to_file(output_path, &processed_data)?;
    
    let stats = processor.calculate_statistics();
    println!("Statistics - Average: {:.2}, Min: {:.2}, Max: {:.2}", 
             stats.0, stats.1, stats.2);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, "CATEGORY".to_string());
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = CsvRecord::new(2, "".to_string(), -10.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_csv_record_transformation() {
        let mut record = CsvRecord::new(1, "test name".to_string(), 123.456, "CATEGORY".to_string());
        record.transform();
        
        assert_eq!(record.name, "TEST NAME");
        assert_eq!(record.category, "category");
        assert_eq!(record.value, 123.46);
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut temp_input = NamedTempFile::new()?;
        writeln!(temp_input, "ID,NAME,VALUE,CATEGORY")?;
        writeln!(temp_input, "1,Product A,100.50,Electronics")?;
        writeln!(temp_input, "2,Product B,75.25,Books")?;
        
        let temp_output = NamedTempFile::new()?;
        
        let result = process_csv_data(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap()
        );
        
        assert!(result.is_ok());
        Ok(())
    }
}