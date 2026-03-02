
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    data: Vec<Vec<String>>,
    headers: Vec<String>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };
        
        let mut data = Vec::new();
        for line in lines {
            let line = line?;
            let row: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if row.len() == headers.len() {
                data.push(row);
            }
        }
        
        Ok(CsvProcessor { data, headers })
    }
    
    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };
        
        self.data
            .iter()
            .filter(|row| row.get(column_index).map_or(false, |cell| cell == value))
            .cloned()
            .collect()
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str) -> Result<f64, Box<dyn Error>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Err("Column not found".into()),
        };
        
        let mut sum = 0.0;
        let mut count = 0;
        
        for row in &self.data {
            if let Some(cell) = row.get(column_index) {
                if let Ok(value) = cell.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Err("No numeric values found".into())
        }
    }
    
    pub fn get_row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Alice,35,60000").unwrap();
        
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.get_row_count(), 3);
        assert_eq!(processor.get_headers(), &vec!["name".to_string(), "age".to_string(), "salary".to_string()]);
        
        let alice_rows = processor.filter_by_column("name", "Alice");
        assert_eq!(alice_rows.len(), 2);
        
        let avg_salary = processor.aggregate_numeric_column("salary").unwrap();
        assert!((avg_salary - 51666.666).abs() < 0.001);
    }
}