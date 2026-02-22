use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvData {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvData {
    pub fn new(headers: Vec<String>, records: Vec<Vec<String>>) -> Self {
        CsvData { headers, records }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.headers.is_empty() {
            return Err("CSV headers cannot be empty".to_string());
        }

        for (i, record) in self.records.iter().enumerate() {
            if record.len() != self.headers.len() {
                return Err(format!(
                    "Record {} has {} fields, expected {}",
                    i + 1,
                    record.len(),
                    self.headers.len()
                ));
            }
        }
        Ok(())
    }

    pub fn get_column(&self, column_name: &str) -> Option<Vec<&str>> {
        let index = self.headers.iter().position(|h| h == column_name)?;
        Some(self.records.iter().map(|r| r[index].as_str()).collect())
    }
}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<CsvData, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
    
    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result?;
        records.push(record.iter().map(|s| s.to_string()).collect());
    }
    
    let csv_data = CsvData::new(headers, records);
    csv_data.validate()?;
    
    Ok(csv_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let result = parse_csv_file(temp_file.path());
        assert!(result.is_ok());
        
        let csv_data = result.unwrap();
        assert_eq!(csv_data.headers, vec!["name", "age", "city"]);
        assert_eq!(csv_data.records.len(), 2);
    }

    #[test]
    fn test_get_column() {
        let headers = vec!["name".to_string(), "age".to_string()];
        let records = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];
        
        let csv_data = CsvData::new(headers, records);
        let ages = csv_data.get_column("age").unwrap();
        assert_eq!(ages, vec!["30", "25"]);
    }
}