
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

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub fn read_csv_file(file_path: &Path) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>().unwrap_or_default();
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>().unwrap_or_default();
            let category = parts[3].to_string();

            let record = CsvRecord::new(id, name, value, category);
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_valid_records(records: Vec<CsvRecord>) -> Vec<CsvRecord> {
    records.into_iter().filter(|r| r.is_valid()).collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn group_by_category(records: Vec<CsvRecord>) -> std::collections::HashMap<String, Vec<CsvRecord>> {
    let mut grouped = std::collections::HashMap::new();
    
    for record in records {
        grouped.entry(record.category.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 10.5, "CategoryA".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = CsvRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_read_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.0,CategoryB").unwrap();

        let records = read_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Item1");
        assert_eq!(records[1].value, 20.0);
    }

    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            CsvRecord::new(1, "Item1".to_string(), 10.0, "CatA".to_string()),
            CsvRecord::new(2, "Item2".to_string(), 20.0, "CatB".to_string()),
            CsvRecord::new(3, "Item3".to_string(), 30.0, "CatA".to_string()),
        ];

        let total = calculate_total_value(&records);
        assert_eq!(total, 60.0);
    }

    #[test]
    fn test_group_by_category() {
        let records = vec![
            CsvRecord::new(1, "Item1".to_string(), 10.0, "CategoryA".to_string()),
            CsvRecord::new(2, "Item2".to_string(), 20.0, "CategoryB".to_string()),
            CsvRecord::new(3, "Item3".to_string(), 30.0, "CategoryA".to_string()),
        ];

        let grouped = group_by_category(records);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get("CategoryA").unwrap().len(), 2);
        assert_eq!(grouped.get("CategoryB").unwrap().len(), 1);
    }
}