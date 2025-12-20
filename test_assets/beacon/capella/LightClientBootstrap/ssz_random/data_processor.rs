
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let mut records = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            
            if Self::validate_record(&fields) {
                records.push(fields);
            } else {
                eprintln!("Warning: Skipping invalid record: {:?}", fields);
            }
        }
        
        Ok(records)
    }
    
    fn validate_record(fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|f| !f.trim().is_empty())
    }
    
    pub fn calculate_statistics(records: &[Vec<String>]) -> (usize, usize) {
        let total_records = records.len();
        let total_fields = records.iter().map(|r| r.len()).sum();
        (total_records, total_fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let records = processor.process().unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0][0], "John");
        assert_eq!(records[1][2], "London");
        
        let stats = DataProcessor::calculate_statistics(&records);
        assert_eq!(stats, (2, 6));
    }
    
    #[test]
    fn test_invalid_records() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "field1,field2").unwrap();
        writeln!(temp_file, "value1,").unwrap();
        writeln!(temp_file, ",value2").unwrap();
        writeln!(temp_file, "valid1,valid2").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let records = processor.process().unwrap();
        
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], vec!["valid1", "valid2"]);
    }
}