use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub fn parse_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
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
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(format!("Empty name field at line {}", line_number).into());
        }

        let value = fields[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(format!("Invalid boolean value at line {}", line_number).into()),
        };

        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }

    Ok(records)
}

pub fn validate_records(records: &[CsvRecord]) -> Result<(), Box<dyn Error>> {
    let mut seen_ids = std::collections::HashSet::new();
    
    for record in records {
        if seen_ids.contains(&record.id) {
            return Err(format!("Duplicate ID found: {}", record.id).into());
        }
        seen_ids.insert(record.id);
        
        if record.value < 0.0 {
            return Err(format!("Negative value for ID {}: {}", record.id, record.value).into());
        }
        
        if record.name.len() > 100 {
            return Err(format!("Name too long for ID {}: {} characters", record.id, record.name.len()).into());
        }
    }
    
    Ok(())
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, usize) {
    let active_records: Vec<&CsvRecord> = records.iter()
        .filter(|r| r.active)
        .collect();
    
    if active_records.is_empty() {
        return (0.0, 0.0, 0);
    }
    
    let sum: f64 = active_records.iter().map(|r| r.value).sum();
    let count = active_records.len();
    let average = sum / count as f64;
    
    let variance: f64 = active_records.iter()
        .map(|r| (r.value - average).powi(2))
        .sum::<f64>() / count as f64;
    
    (average, variance, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,42.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,17.3,false").unwrap();
        writeln!(temp_file, "3,ItemC,89.1,yes").unwrap();
        
        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 17.3);
        assert!(records[2].active);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            CsvRecord { id: 1, name: "Test".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
        ];
        
        assert!(validate_records(&records).is_ok());
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: true },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: false },
        ];
        
        let (avg, var, count) = calculate_statistics(&records);
        assert_eq!(count, 2);
        assert_eq!(avg, 15.0);
        assert_eq!(var, 25.0);
    }
}