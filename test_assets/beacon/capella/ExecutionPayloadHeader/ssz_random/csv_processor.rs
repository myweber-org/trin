use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(String),
    ParseError(String, usize),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO error: {}", msg),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(&path).map_err(|e| CsvError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        
        self.records.clear();
        
        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(|e| CsvError::IoError(e.to_string()))?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if fields.len() != 4 {
                return Err(CsvError::ParseError(
                    format!("Expected 4 fields, found {}", fields.len()),
                    line_num + 1,
                ));
            }
            
            let id = fields[0].parse::<u32>()
                .map_err(|_| CsvError::ParseError(
                    format!("Invalid ID: {}", fields[0]),
                    line_num + 1,
                ))?;
            
            let name = fields[1].to_string();
            
            let value = fields[2].parse::<f64>()
                .map_err(|_| CsvError::ParseError(
                    format!("Invalid value: {}", fields[2]),
                    line_num + 1,
                ))?;
            
            let active = match fields[3].to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                "false" | "0" | "no" => false,
                _ => return Err(CsvError::ParseError(
                    format!("Invalid boolean: {}", fields[3]),
                    line_num + 1,
                )),
            };
            
            self.records.push(CsvRecord {
                id,
                name,
                value,
                active,
            });
        }
        
        self.validate_records()?;
        
        Ok(())
    }
    
    fn validate_records(&self) -> Result<(), CsvError> {
        let mut seen_ids = std::collections::HashSet::new();
        
        for record in &self.records {
            if !seen_ids.insert(record.id) {
                return Err(CsvError::ValidationError(
                    format!("Duplicate ID found: {}", record.id)
                ));
            }
            
            if record.value < 0.0 {
                return Err(CsvError::ValidationError(
                    format!("Negative value not allowed for ID: {}", record.id)
                ));
            }
            
            if record.name.is_empty() {
                return Err(CsvError::ValidationError(
                    format!("Empty name for ID: {}", record.id)
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn get_active_records(&self) -> Vec<&CsvRecord> {
        self.records.iter().filter(|r| r.active).collect()
    }
    
    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }
    
    pub fn find_by_id(&self, id: u32) -> Option<&CsvRecord> {
        self.records.iter().find(|r| r.id == id)
    }
    
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_loading() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,ItemA,10.5,true").unwrap();
        writeln!(csv_data, "2,ItemB,20.0,false").unwrap();
        writeln!(csv_data, "3,ItemC,15.75,true").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.calculate_total_value(), 46.25);
        assert_eq!(processor.get_active_records().len(), 2);
    }
    
    #[test]
    fn test_duplicate_id_validation() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,ItemA,10.5,true").unwrap();
        writeln!(csv_data, "1,ItemB,20.0,false").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_err());
        if let Err(CsvError::ValidationError(msg)) = result {
            assert!(msg.contains("Duplicate ID"));
        } else {
            panic!("Expected ValidationError");
        }
    }
    
    #[test]
    fn test_find_by_id() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "100,TestItem,42.0,true").unwrap();
        
        let mut processor = CsvProcessor::new();
        processor.load_from_file(csv_data.path()).unwrap();
        
        let found = processor.find_by_id(100);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "TestItem");
        
        let not_found = processor.find_by_id(999);
        assert!(not_found.is_none());
    }
}