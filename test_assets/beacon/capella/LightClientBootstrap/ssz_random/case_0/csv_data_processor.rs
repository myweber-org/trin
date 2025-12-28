use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
struct CsvRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl CsvRecord {
    fn from_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(CsvRecord {
            id: parts[0].parse()?,
            category: parts[1].to_string(),
            value: parts[2].parse()?,
            active: parts[3].parse()?,
        })
    }
}

struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
            let record = CsvRecord::from_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn aggregate_by_category(&self) -> HashMap<String, f64> {
        let mut aggregates = HashMap::new();

        for record in &self.records {
            if record.active {
                let entry = aggregates.entry(record.category.clone()).or_insert(0.0);
                *entry += record.value;
            }
        }

        aggregates
    }

    fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }
}

fn process_csv_data() -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    processor.load_from_file("data.csv")?;

    let electronics = processor.filter_by_category("electronics");
    println!("Found {} electronics records", electronics.len());

    let aggregates = processor.aggregate_by_category();
    for (category, total) in aggregates {
        println!("Category {}: total value {}", category, total);
    }

    if let Some(max_record) = processor.find_max_value() {
        println!("Maximum value record: {:?}", max_record);
    }

    Ok(())
}

fn main() {
    if let Err(e) = process_csv_data() {
        eprintln!("Error processing CSV: {}", e);
    }
}