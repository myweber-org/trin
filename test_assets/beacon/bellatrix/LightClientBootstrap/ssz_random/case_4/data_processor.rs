
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
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            
            if Self::validate_row(&row) {
                records.push(row);
            } else {
                eprintln!("Warning: Skipping invalid row: {:?}", row);
            }
        }
        
        Ok(records)
    }

    fn validate_row(row: &[String]) -> bool {
        !row.is_empty() && row.iter().all(|field| !field.trim().is_empty())
    }

    pub fn calculate_statistics(data: &[Vec<String>]) -> (usize, usize) {
        let total_rows = data.len();
        let total_fields = data.iter().map(|row| row.len()).sum();
        (total_rows, total_fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "John");
        assert_eq!(result[1][2], "London");
    }

    #[test]
    fn test_statistics_calculation() {
        let test_data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string(), "e".to_string()],
        ];
        
        let (rows, fields) = DataProcessor::calculate_statistics(&test_data);
        assert_eq!(rows, 2);
        assert_eq!(fields, 5);
    }
}