
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 5 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            category: parts[2].to_string(),
            value: parts[3].parse()?,
            active: parts[4].parse()?,
        })
    }
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let record = Record::from_csv_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    fn summary_statistics(&self) -> String {
        let avg = self.calculate_average_value();
        let max_record = self.find_max_value();
        let active_count = self.filter_active().len();
        let category_groups = self.group_by_category();
        
        let mut summary = format!(
            "Total records: {}\nAverage value: {:.2}\nActive records: {}\n",
            self.records.len(),
            avg,
            active_count
        );
        
        if let Some(max) = max_record {
            summary.push_str(&format!("Max value: {} (ID: {})\n", max.value, max.id));
        }
        
        summary.push_str("Records by category:\n");
        for (category, records) in category_groups {
            summary.push_str(&format!("  {}: {}\n", category, records.len()));
        }
        
        summary
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    match processor.load_from_file("data.csv") {
        Ok(_) => {
            println!("Data loaded successfully");
            println!("{}", processor.summary_statistics());
            
            let electronics = processor.filter_by_category("electronics");
            println!("Electronics records: {}", electronics.len());
            
            for record in electronics.iter().take(3) {
                println!("  - {}: ${:.2}", record.name, record.value);
            }
        }
        Err(e) => {
            eprintln!("Error loading data: {}", e);
            println!("Creating sample data for demonstration...");
            
            processor.records = vec![
                Record {
                    id: 1,
                    name: "Laptop".to_string(),
                    category: "electronics".to_string(),
                    value: 999.99,
                    active: true,
                },
                Record {
                    id: 2,
                    name: "Desk".to_string(),
                    category: "furniture".to_string(),
                    value: 299.50,
                    active: true,
                },
                Record {
                    id: 3,
                    name: "Monitor".to_string(),
                    category: "electronics".to_string(),
                    value: 199.99,
                    active: false,
                },
                Record {
                    id: 4,
                    name: "Chair".to_string(),
                    category: "furniture".to_string(),
                    value: 149.99,
                    active: true,
                },
            ];
            
            println!("{}", processor.summary_statistics());
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            category: "test".to_string(),
            value: 100.0,
            active: true,
        };
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test");
        assert_eq!(record.value, 100.0);
    }

    #[test]
    fn test_filter_active() {
        let mut processor = DataProcessor::new();
        processor.records = vec![
            Record {
                id: 1,
                name: "Item1".to_string(),
                category: "cat1".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Item2".to_string(),
                category: "cat2".to_string(),
                value: 20.0,
                active: false,
            },
        ];
        
        let active = processor.filter_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records = vec![
            Record {
                id: 1,
                name: "A".to_string(),
                category: "cat".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "B".to_string(),
                category: "cat".to_string(),
                value: 20.0,
                active: true,
            },
        ];
        
        assert_eq!(processor.calculate_average_value(), 15.0);
    }
}