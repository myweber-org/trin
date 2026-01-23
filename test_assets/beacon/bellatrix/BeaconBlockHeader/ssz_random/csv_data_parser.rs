use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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
}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line_content.split(',').collect();
        
        if fields.len() != 4 {
            return Err(format!("Invalid field count at line {}", line_number).into());
        }

        let id = fields[0].parse::<u32>()
            .map_err(|_| format!("Invalid ID at line {}", line_number))?;
        
        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(format!("Empty name at line {}", line_number).into());
        }

        let value = fields[2].parse::<f64>()
            .map_err(|_| format!("Invalid value at line {}", line_number))?;
        
        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(format!("Invalid boolean at line {}", line_number).into()),
        };

        records.push(Record::new(id, name, value, active));
    }

    if records.is_empty() {
        return Err("No valid records found in file".into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let active_count = records.iter().filter(|r| r.active).count();
    
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / records.len() as f64;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - avg).powi(2))
        .sum::<f64>() / records.len() as f64;
    
    (avg, variance.sqrt(), active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,yes").unwrap();

        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], Record::new(1, "ItemA".to_string(), 10.5, true));
        assert_eq!(records[2], Record::new(3, "ItemC".to_string(), 15.7, true));
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record::new(1, "Test1".to_string(), 10.0, true),
            Record::new(2, "Test2".to_string(), 20.0, false),
            Record::new(3, "Test3".to_string(), 30.0, true),
        ];
        
        let (avg, std_dev, active_count) = calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert!(std_dev - 8.164965 < 0.0001);
        assert_eq!(active_count, 2);
    }

    #[test]
    fn test_invalid_csv_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,10.5").unwrap(); // Missing field
        
        let result = parse_csv_file(temp_file.path());
        assert!(result.is_err());
    }
}