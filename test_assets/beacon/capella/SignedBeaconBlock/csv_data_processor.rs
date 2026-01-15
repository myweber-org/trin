use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
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

    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(Record {
                id,
                name,
                value,
                category,
            });
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average_value(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let total: f64 = records.iter().map(|record| record.value).sum();
    total / records.len() as f64
}

pub fn find_max_value_record(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;
    
    let mut category_totals: HashMap<String, f64> = HashMap::new();
    
    for record in records {
        *category_totals.entry(record.category.clone()).or_insert(0.0) += record.value;
    }
    
    category_totals.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_csv_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        
        let records = load_csv_data(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 20.3);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];
        
        let filtered = filter_by_category(&records, "Cat1");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "Cat1"));
    }

    #[test]
    fn test_calculate_average_value() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];
        
        let avg = calculate_average_value(&records);
        assert_eq!(avg, 20.0);
    }

    #[test]
    fn test_find_max_value_record() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 30.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 20.0, category: "Cat1".to_string() },
        ];
        
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 30.0);
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Cat2".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Cat1".to_string() },
        ];
        
        let aggregated = aggregate_by_category(&records);
        assert_eq!(aggregated.len(), 2);
        
        let cat1_total: f64 = aggregated.iter()
            .find(|(cat, _)| cat == "Cat1")
            .map(|(_, total)| *total)
            .unwrap_or(0.0);
        
        assert_eq!(cat1_total, 40.0);
    }
}