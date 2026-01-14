
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn read_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
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
            return Err(format!("Invalid number of fields at line {}", line_number).into());
        }
        
        let id = fields[0].parse::<u32>()
            .map_err(|_| format!("Invalid ID at line {}", line_number))?;
        
        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(format!("Empty name at line {}", line_number).into());
        }
        
        let value = fields[2].parse::<f64>()
            .map_err(|_| format!("Invalid value at line {}", line_number))?;
        
        let category = fields[3].trim().to_string();
        if category.is_empty() {
            return Err(format!("Empty category at line {}", line_number).into());
        }
        
        records.push(CsvRecord {
            id,
            name,
            value,
            category,
        });
    }
    
    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<CsvRecord> {
    records.iter()
        .filter(|record| record.category == category)
        .cloned()
        .collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter()
        .map(|record| record.value)
        .sum()
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter()
        .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn transform_values<F>(records: &mut [CsvRecord], transform_fn: F)
where
    F: Fn(f64) -> f64,
{
    for record in records.iter_mut() {
        record.value = transform_fn(record.value);
    }
}

pub fn validate_records(records: &[CsvRecord]) -> Result<(), String> {
    let mut seen_ids = std::collections::HashSet::new();
    
    for record in records {
        if record.name.len() > 100 {
            return Err(format!("Name too long for record ID {}", record.id));
        }
        
        if record.value < 0.0 {
            return Err(format!("Negative value for record ID {}", record.id));
        }
        
        if !seen_ids.insert(record.id) {
            return Err(format!("Duplicate ID found: {}", record.id));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_read_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Item A,25.5,Electronics").unwrap();
        writeln!(temp_file, "2,Item B,15.0,Books").unwrap();
        writeln!(temp_file, "3,Item C,30.75,Electronics").unwrap();
        
        let records = read_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Item A");
        assert_eq!(records[1].category, "Books");
    }
    
    #[test]
    fn test_filter_by_category() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, category: "A".to_string() },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, category: "B".to_string() },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, category: "A".to_string() },
        ];
        
        let filtered = filter_by_category(&records, "A");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "A"));
    }
    
    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.5, category: "A".to_string() },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.5, category: "B".to_string() },
        ];
        
        let total = calculate_total_value(&records);
        assert_eq!(total, 31.0);
    }
    
    #[test]
    fn test_validate_records() {
        let valid_records = vec![
            CsvRecord { id: 1, name: "Valid".to_string(), value: 10.0, category: "A".to_string() },
            CsvRecord { id: 2, name: "Another".to_string(), value: 20.0, category: "B".to_string() },
        ];
        
        assert!(validate_records(&valid_records).is_ok());
        
        let invalid_records = vec![
            CsvRecord { id: 1, name: "Test".to_string(), value: 10.0, category: "A".to_string() },
            CsvRecord { id: 1, name: "Duplicate".to_string(), value: 20.0, category: "B".to_string() },
        ];
        
        assert!(validate_records(&invalid_records).is_err());
    }
}