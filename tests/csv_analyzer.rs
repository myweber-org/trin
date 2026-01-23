
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub has_headers: bool,
    pub sample_data: Vec<Vec<String>>,
}

pub fn analyze_csv<P: AsRef<Path>>(file_path: P, sample_size: usize) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers = rdr.headers()?.clone();
    let has_headers = !headers.is_empty();
    let column_count = headers.len();
    
    let mut row_count = 0;
    let mut sample_data = Vec::with_capacity(sample_size.min(10));
    
    for result in rdr.records().take(sample_size) {
        let record = result?;
        let row_data: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        sample_data.push(row_data);
        row_count += 1;
    }
    
    row_count += rdr.records().count();
    
    Ok(CsvStats {
        row_count,
        column_count,
        has_headers,
        sample_data,
    })
}

pub fn validate_csv_data<P: AsRef<Path>>(file_path: P) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut errors = Vec::new();
    
    for (index, result) in rdr.records().enumerate() {
        match result {
            Ok(record) => {
                for (col_idx, field) in record.iter().enumerate() {
                    if field.trim().is_empty() {
                        errors.push(format!("Row {} column {} contains empty value", index + 1, col_idx + 1));
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Error reading row {}: {}", index + 1, e));
            }
        }
    }
    
    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let stats = analyze_csv(temp_file.path(), 2).unwrap();
        assert_eq!(stats.row_count, 2);
        assert_eq!(stats.column_count, 3);
        assert!(stats.has_headers);
        assert_eq!(stats.sample_data.len(), 2);
    }
    
    #[test]
    fn test_validate_csv_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,").unwrap();
        writeln!(temp_file, ",25,London").unwrap();
        
        let errors = validate_csv_data(temp_file.path()).unwrap();
        assert_eq!(errors.len(), 2);
    }
}