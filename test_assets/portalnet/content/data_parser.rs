use csv::{Reader, StringRecord};
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DataRow {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataParser {
    reader: Reader<File>,
}

impl DataParser {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = csv::Reader::from_reader(file);
        Ok(DataParser { reader })
    }

    pub fn parse_records(&mut self) -> Result<Vec<DataRow>, Box<dyn Error>> {
        let mut records = Vec::new();
        
        for result in self.reader.deserialize() {
            let record: DataRow = result?;
            records.push(record);
        }
        
        Ok(records)
    }

    pub fn parse_with_validation(&mut self) -> Result<Vec<DataRow>, Box<dyn Error>> {
        let mut valid_records = Vec::new();
        
        for (index, result) in self.reader.deserialize().enumerate() {
            match result {
                Ok(record) => {
                    if record.value >= 0.0 {
                        valid_records.push(record);
                    } else {
                        eprintln!("Warning: Negative value at row {}", index + 1);
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing row {}: {}", index + 1, e);
                }
            }
        }
        
        Ok(valid_records)
    }

    pub fn get_headers(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        let headers = self.reader.headers()?;
        Ok(headers.iter().map(|s| s.to_string()).collect())
    }
}

pub fn calculate_statistics(records: &[DataRow]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / records.len() as f64;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / records.len() as f64;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,active").unwrap();
        writeln!(file, "1,ItemA,10.5,true").unwrap();
        writeln!(file, "2,ItemB,20.3,false").unwrap();
        writeln!(file, "3,ItemC,15.7,true").unwrap();
        file
    }

    #[test]
    fn test_parse_records() {
        let test_file = create_test_csv();
        let mut parser = DataParser::new(test_file.path()).unwrap();
        let records = parser.parse_records().unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 20.3);
        assert!(records[2].active);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRow { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            DataRow { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            DataRow { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!(std_dev > 8.16 && std_dev < 8.17);
    }
}