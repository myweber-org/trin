use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
struct CsvRecord {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl CsvRecord {
    fn from_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid number of fields".into());
        }

        let id = parts[0].parse()?;
        let name = parts[1].to_string();
        let value = parts[2].parse()?;
        let active = parts[3].parse()?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }
}

fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        match CsvRecord::from_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Error parsing line {}: {}", line_num + 1, e),
        }
    }

    Ok(records)
}

fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, usize) {
    let total: f64 = records.iter().map(|r| r.value).sum();
    let average = total / records.len() as f64;
    let active_count = records.iter().filter(|r| r.active).count();

    (total, average, active_count)
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = process_csv_file("data.csv")?;
    
    if records.is_empty() {
        println!("No valid records found in CSV file");
        return Ok(());
    }

    let (total, average, active_count) = calculate_statistics(&records);
    
    println!("Processed {} records", records.len());
    println!("Total value: {:.2}", total);
    println!("Average value: {:.2}", average);
    println!("Active records: {}", active_count);
    
    for record in records.iter().take(3) {
        println!("Sample record: {:?}", record);
    }

    Ok(())
}