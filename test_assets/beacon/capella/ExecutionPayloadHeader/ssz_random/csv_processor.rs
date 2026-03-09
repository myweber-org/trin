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
}use std::error::Error;
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
            CsvError::IoError(msg) => write!(f, "IO Error: {}", msg),
            CsvError::ParseError(msg, line) => write!(f, "Parse Error at line {}: {}", line, msg),
            CsvError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
    total_value: f64,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
            total_value: 0.0,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(&path).map_err(|e| CsvError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| CsvError::IoError(e.to_string()))?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let record = self.parse_line(&line, line_num + 1)?;
            self.validate_record(&record)?;
            self.records.push(record);
        }
        
        self.calculate_total();
        Ok(())
    }
    
    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Expected 4 columns, found {}", parts.len()),
                line_num
            ));
        }
        
        let id = parts[0].parse::<u32>()
            .map_err(|_| CsvError::ParseError(
                format!("Invalid ID: {}", parts[0]),
                line_num
            ))?;
        
        let name = parts[1].to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|_| CsvError::ParseError(
                format!("Invalid value: {}", parts[2]),
                line_num
            ))?;
        
        let active = parts[3].parse::<bool>()
            .map_err(|_| CsvError::ParseError(
                format!("Invalid boolean: {}", parts[3]),
                line_num
            ))?;
        
        Ok(CsvRecord { id, name, value, active })
    }
    
    fn validate_record(&self, record: &CsvRecord) -> Result<(), CsvError> {
        if record.name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Record {} has empty name", record.id)
            ));
        }
        
        if record.value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Record {} has negative value: {}", record.id, record.value)
            ));
        }
        
        Ok(())
    }
    
    fn calculate_total(&mut self) {
        self.total_value = self.records.iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .sum();
    }
    
    pub fn get_active_records(&self) -> Vec<&CsvRecord> {
        self.records.iter()
            .filter(|r| r.active)
            .collect()
    }
    
    pub fn get_total_value(&self) -> f64 {
        self.total_value
    }
    
    pub fn find_by_id(&self, id: u32) -> Option<&CsvRecord> {
        self.records.iter().find(|r| r.id == id)
    }
    
    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let active_values: Vec<f64> = self.records.iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .collect();
        
        if active_values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let count = active_values.len() as f64;
        let sum: f64 = active_values.iter().sum();
        let avg = sum / count;
        
        let variance: f64 = active_values.iter()
            .map(|&v| (v - avg).powi(2))
            .sum::<f64>() / count;
        
        (sum, avg, variance.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_processing() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "# Test CSV file").unwrap();
        writeln!(csv_data, "1,ItemA,25.5,true").unwrap();
        writeln!(csv_data, "2,ItemB,30.0,true").unwrap();
        writeln!(csv_data, "3,ItemC,15.75,false").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.get_active_records().len(), 2);
        assert_eq!(processor.get_total_value(), 55.5);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 55.5);
        assert_eq!(stats.1, 27.75);
    }
    
    #[test]
    fn test_invalid_csv() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,ItemA,invalid,true").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_err());
    }
}