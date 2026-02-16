
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

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[3].to_string();
            
            self.records.push(CsvRecord {
                id,
                name,
                value,
                category,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processor() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "id,name,value,category").unwrap();
        writeln!(csv_data, "1,ItemA,10.5,Category1").unwrap();
        writeln!(csv_data, "2,ItemB,15.2,Category2").unwrap();
        writeln!(csv_data, "3,ItemC,8.7,Category1").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.get_records().len(), 3);
        
        let category1_items = processor.filter_by_category("Category1");
        assert_eq!(category1_items.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().name, "ItemB");
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }
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
        if parts.len() == 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(Record::new(id, name, value, category));
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

pub fn calculate_average(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record::new(1, "ItemA".to_string(), 10.5, "Alpha".to_string()),
            Record::new(2, "ItemB".to_string(), 20.3, "Beta".to_string()),
            Record::new(3, "ItemC".to_string(), 15.7, "Alpha".to_string()),
        ];

        let filtered = filter_by_category(&records, "Alpha");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record::new(1, "Test1".to_string(), 10.0, "Cat1".to_string()),
            Record::new(2, "Test2".to_string(), 20.0, "Cat1".to_string()),
        ];

        let refs: Vec<&Record> = records.iter().collect();
        let avg = calculate_average(&refs);
        assert_eq!(avg, Some(15.0));
    }
}