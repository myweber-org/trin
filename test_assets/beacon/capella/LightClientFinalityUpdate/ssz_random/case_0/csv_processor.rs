
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

fn parse_csv_line(line: &str) -> Result<Record, Box<dyn Error>> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() != 4 {
        return Err("Invalid number of fields".into());
    }

    let id = parts[0].parse::<u32>()?;
    let name = parts[1].trim().to_string();
    let value = parts[2].parse::<f64>()?;
    let active = parts[3].parse::<bool>()?;

    let record = Record {
        id,
        name,
        value,
        active,
    };

    record.validate()?;
    Ok(record)
}

fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match parse_csv_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Error parsing line {}: {}", line_num + 1, e),
        }
    }

    Ok(records)
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let avg = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let active_count = records.iter().filter(|r| r.active).count();
    
    (sum, avg, active_count)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <csv_file>", args[0]);
        std::process::exit(1);
    }

    let records = process_csv_file(&args[1])?;
    
    println!("Processed {} records", records.len());
    
    let (total, average, active_count) = calculate_statistics(&records);
    println!("Total value: {:.2}", total);
    println!("Average value: {:.2}", average);
    println!("Active records: {}", active_count);
    
    for record in records.iter().take(5) {
        println!("{:?}", record);
    }
    
    Ok(())
}