use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid format at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid ID at line {}", line_number))?;
        
        let name = parts[1].trim().to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|_| format!("Invalid value at line {}", line_number))?;
        
        let active = parts[3].trim().parse::<bool>()
            .map_err(|_| format!("Invalid boolean at line {}", line_number))?;

        let record = Record::new(id, name, value, active);
        if !record.is_valid() {
            return Err(format!("Invalid record data at line {}", line_number).into());
        }

        records.push(record);
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let active_records: Vec<&Record> = records.iter()
        .filter(|r| r.active)
        .collect();

    if active_records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = active_records.iter()
        .map(|r| r.value)
        .sum();
    
    let count = active_records.len();
    let average = sum / count as f64;
    
    let variance: f64 = active_records.iter()
        .map(|r| (r.value - average).powi(2))
        .sum::<f64>() / count as f64;
    
    (average, variance.sqrt(), count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,20.0,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "3,ItemC,15.75,true").unwrap();

        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].name, "ItemB");
    }

    #[test]
    fn test_invalid_csv_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,10.5").unwrap();

        let result = parse_csv_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record::new(1, "Test1".to_string(), 10.0, true),
            Record::new(2, "Test2".to_string(), 20.0, true),
            Record::new(3, "Test3".to_string(), 30.0, false),
        ];

        let (avg, std_dev, count) = calculate_statistics(&records);
        assert_eq!(count, 2);
        assert_eq!(avg, 15.0);
        assert!(std_dev > 0.0);
    }
}