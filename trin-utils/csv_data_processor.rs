use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
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
        if name.is_empty() {
            return Err(format!("Empty name at line {}", line_number).into());
        }

        let value = parts[2].parse::<f64>()
            .map_err(|_| format!("Invalid value at line {}", line_number))?;
        
        let category = parts[3].trim().to_string();
        if category.is_empty() {
            return Err(format!("Empty category at line {}", line_number).into());
        }

        records.push(Record {
            id,
            name,
            value,
            category,
        });
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<Record> {
    records.iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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

pub fn transform_values<F>(records: &mut [Record], transform_fn: F)
where
    F: Fn(f64) -> f64,
{
    for record in records.iter_mut() {
        record.value = transform_fn(record.value);
    }
}

pub fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.len() > 100 {
        return Err("Name too long".to_string());
    }
    
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    
    if record.category.len() > 50 {
        return Err("Category too long".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_csv_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Product A,25.5,Electronics").unwrap();
        writeln!(temp_file, "2,Product B,30.0,Books").unwrap();
        writeln!(temp_file, "3,Product C,15.75,Electronics").unwrap();

        let records = load_csv_data(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Product A");
        assert_eq!(records[1].category, "Books");
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Y".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "X".to_string() },
        ];

        let filtered = filter_by_category(&records, "X");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "X".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "X".to_string() },
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "Category".to_string(),
        };
        assert!(validate_record(&valid_record).is_ok());

        let invalid_record = Record {
            id: 2,
            name: "A".repeat(101),
            value: -5.0,
            category: "C".to_string(),
        };
        assert!(validate_record(&invalid_record).is_err());
    }
}