use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 4 {
            continue;
        }

        let record = Record {
            id: parts[0].parse()?,
            category: parts[1].to_string(),
            value: parts[2].parse()?,
            active: parts[3].parse().unwrap_or(false),
        };
        records.push(record);
    }

    Ok(records)
}

pub fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

pub fn calculate_average_value(records: &[Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn group_by_category(records: &[Record]) -> std::collections::HashMap<String, Vec<&Record>> {
    let mut groups = std::collections::HashMap::new();
    
    for record in records {
        groups
            .entry(record.category.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,299.99,true").unwrap();
        writeln!(temp_file, "2,books,19.99,false").unwrap();
        
        let records = load_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].category, "electronics");
    }

    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record { id: 1, category: String::from("a"), value: 10.0, active: true },
            Record { id: 2, category: String::from("b"), value: 20.0, active: false },
            Record { id: 3, category: String::from("c"), value: 30.0, active: true },
        ];
        
        let active = filter_active_records(&records);
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record { id: 1, category: String::from("a"), value: 10.0, active: true },
            Record { id: 2, category: String::from("b"), value: 20.0, active: true },
            Record { id: 3, category: String::from("c"), value: 30.0, active: true },
        ];
        
        let avg = calculate_average_value(&records).unwrap();
        assert_eq!(avg, 20.0);
    }
}