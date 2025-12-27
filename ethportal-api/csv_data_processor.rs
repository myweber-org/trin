use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn read_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
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
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();
            
            records.push(CsvRecord {
                id,
                name,
                value,
                category,
            });
        }
    }
    
    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<&CsvRecord> {
    records.iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average_value(records: &[CsvRecord]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    
    let total: f64 = records.iter().map(|r| r.value).sum();
    total / records.len() as f64
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_processing() {
        let csv_data = "id,name,value,category\n\
                        1,ItemA,25.5,Electronics\n\
                        2,ItemB,42.8,Books\n\
                        3,ItemC,18.3,Electronics\n\
                        4,ItemD,99.9,Books";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let records = read_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 4);
        
        let electronics = filter_by_category(&records, "Electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = calculate_average_value(&records);
        assert!((avg - 46.625).abs() < 0.001);
        
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 4);
        assert_eq!(max_record.value, 99.9);
    }
}