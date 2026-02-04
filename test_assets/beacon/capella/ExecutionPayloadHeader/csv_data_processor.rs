use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            value: parts[2].parse()?,
            category: parts[3].to_string(),
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

    fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
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

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        use std::collections::HashMap;

        let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        aggregates
            .into_iter()
            .map(|(category, (total, count))| (category, total / count as f64))
            .collect()
    }
}

fn process_data_file() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file("data.csv")?;
    
    println!("Total records loaded: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Maximum value record: {:?}", max_record);
    }
    
    let tech_records = processor.filter_by_category("Technology");
    println!("Technology records: {}", tech_records.len());
    
    let aggregates = processor.aggregate_by_category();
    for (category, avg) in aggregates {
        println!("Category {} average: {:.2}", category, avg);
    }
    
    Ok(())
}

fn main() {
    if let Err(e) = process_data_file() {
        eprintln!("Error processing data: {}", e);
    }
}