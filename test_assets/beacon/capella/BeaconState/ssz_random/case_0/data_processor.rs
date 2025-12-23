
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: String) -> Self {
        DataRecord {
            id,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.timestamp.is_empty()
    }
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let mut records = Vec::new();
    
    for result in rdr.deserialize() {
        let record: DataRecord = result?;
        if record.is_valid() {
            records.push(record);
        }
    }
    
    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, "2024-01-15T10:30:00Z".to_string());
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, f64::NAN, "".to_string());
        assert!(!record.is_valid());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 10.0, "2024-01-15T10:30:00Z".to_string()),
            DataRecord::new(2, 20.0, "2024-01-15T11:30:00Z".to_string()),
            DataRecord::new(3, 30.0, "2024-01-15T12:30:00Z".to_string()),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert!((std_dev - 8.16496580927726).abs() < 1e-10);
    }

    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value,timestamp")?;
        writeln!(temp_file, "1,42.5,2024-01-15T10:30:00Z")?;
        writeln!(temp_file, "2,37.8,2024-01-15T11:30:00Z")?;
        
        let records = load_csv_data(temp_file.path().to_str().unwrap())?;
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].value, 37.8);
        
        Ok(())
    }
}