
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub has_header: bool,
    pub sample_data: Vec<Vec<String>>,
}

pub fn analyze_csv<P: AsRef<Path>>(file_path: P, sample_size: usize) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers = rdr.headers()?.clone();
    let has_header = !headers.is_empty();
    let column_count = headers.len();
    
    let mut row_count = 0;
    let mut sample_data = Vec::with_capacity(sample_size);
    
    for result in rdr.records() {
        let record = result?;
        row_count += 1;
        
        if sample_data.len() < sample_size {
            let row_data: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            sample_data.push(row_data);
        }
    }
    
    Ok(CsvStats {
        row_count,
        column_count,
        has_header,
        sample_data,
    })
}

pub fn validate_csv_structure<P: AsRef<Path>>(file_path: P) -> Result<bool, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers = rdr.headers()?;
    let expected_columns = headers.len();
    
    for (index, result) in rdr.records().enumerate() {
        let record = result?;
        if record.len() != expected_columns {
            return Err(format!("Row {} has {} columns, expected {}", 
                index + 1, record.len(), expected_columns).into());
        }
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,city").unwrap();
        writeln!(file, "Alice,30,New York").unwrap();
        writeln!(file, "Bob,25,London").unwrap();
        writeln!(file, "Charlie,35,Tokyo").unwrap();
        file
    }

    #[test]
    fn test_analyze_csv() {
        let test_file = create_test_csv();
        let stats = analyze_csv(test_file.path(), 2).unwrap();
        
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 3);
        assert!(stats.has_header);
        assert_eq!(stats.sample_data.len(), 2);
    }

    #[test]
    fn test_validate_csv_structure() {
        let test_file = create_test_csv();
        let result = validate_csv_structure(test_file.path());
        assert!(result.is_ok());
    }
}