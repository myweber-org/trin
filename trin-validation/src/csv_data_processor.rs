
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
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

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    category: parts[2].to_string(),
                    value: parts[3].parse()?,
                    active: parts[4].parse().unwrap_or(false),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    pub fn find_max_value_record(&self) -> Option<CsvRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    pub fn find_min_value_record(&self) -> Option<CsvRecord> {
        self.records
            .iter()
            .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.records
            .iter()
            .map(|record| record.category.clone())
            .collect();
        
        categories.sort();
        categories.dedup();
        categories
    }

    pub fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        let mut aggregates = std::collections::HashMap::new();
        
        for record in &self.records {
            *aggregates.entry(record.category.clone()).or_insert(0.0) += record.value;
        }
        
        let mut result: Vec<(String, f64)> = aggregates.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_processor() {
        let processor = CsvProcessor::new();
        assert_eq!(processor.count_records(), 0);
        assert_eq!(processor.calculate_total_value(), 0.0);
        assert_eq!(processor.calculate_average_value(), 0.0);
    }

    #[test]
    fn test_record_creation() {
        let record = CsvRecord {
            id: 1,
            name: "Test Item".to_string(),
            category: "Electronics".to_string(),
            value: 99.99,
            active: true,
        };
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test Item");
        assert_eq!(record.category, "Electronics");
        assert_eq!(record.value, 99.99);
        assert_eq!(record.active, true);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter().position(|h| h == column_name);
        
        column_index.map_or_else(Vec::new, |idx| {
            self.records
                .iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect()
        })
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let sum: f64 = self.records
            .iter()
            .filter_map(|record| record[column_index].parse::<f64>().ok())
            .sum();
        
        Some(sum)
    }

    pub fn get_column_stats(&self, column_name: &str) -> Option<(f64, f64, usize)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse::<f64>().ok())
            .collect();
        
        if values.is_empty() {
            return None;
        }
        
        let sum: f64 = values.iter().sum();
        let count = values.len();
        let average = sum / count as f64;
        
        Some((sum, average, count))
    }

    pub fn get_unique_values(&self, column_name: &str) -> Option<Vec<String>> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let mut unique_values = std::collections::HashSet::new();
        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                unique_values.insert(value.clone());
            }
        }
        
        Some(unique_values.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,salary,department").unwrap();
        writeln!(file, "Alice,30,50000.0,Engineering").unwrap();
        writeln!(file, "Bob,25,45000.0,Marketing").unwrap();
        writeln!(file, "Charlie,35,60000.0,Engineering").unwrap();
        writeln!(file, "Diana,28,48000.0,Sales").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.headers, vec!["name", "age", "salary", "department"]);
        assert_eq!(processor.records.len(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", |dept| dept == "Engineering");
        assert_eq!(engineering_records.len(), 2);
        
        let filtered_names: Vec<String> = engineering_records
            .iter()
            .map(|record| record[0].clone())
            .collect();
        assert!(filtered_names.contains(&"Alice".to_string()));
        assert!(filtered_names.contains(&"Charlie".to_string()));
    }

    #[test]
    fn test_aggregate_numeric() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let total_salary = processor.aggregate_numeric_column("salary").unwrap();
        assert!((total_salary - 203000.0).abs() < 0.001);
    }

    #[test]
    fn test_column_stats() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let stats = processor.get_column_stats("salary").unwrap();
        assert!((stats.0 - 203000.0).abs() < 0.001);
        assert!((stats.1 - 50750.0).abs() < 0.001);
        assert_eq!(stats.2, 4);
    }

    #[test]
    fn test_unique_values() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let departments = processor.get_unique_values("department").unwrap();
        assert_eq!(departments.len(), 3);
        assert!(departments.contains(&"Engineering".to_string()));
        assert!(departments.contains(&"Marketing".to_string()));
        assert!(departments.contains(&"Sales".to_string()));
    }
}