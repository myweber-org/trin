use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return None;
        }

        let id = parts[0].parse().ok()?;
        let name = parts[1].to_string();
        let value = parts[2].parse().ok()?;
        let active = parts[3].parse().ok()?;

        Some(Record {
            id,
            name,
            value,
            active,
        })
    }

    fn to_csv_line(&self) -> String {
        format!("{},{},{},{}", self.id, self.name, self.value, self.active)
    }

    fn filter_by_value(&self, threshold: f64) -> bool {
        self.value > threshold && self.active
    }
}

fn process_csv_file(input_path: &str, output_path: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    let mut records_processed = 0;
    let mut records_written = 0;

    for line_result in reader.lines() {
        let line = line_result?;
        records_processed += 1;

        if let Some(record) = Record::from_csv_line(&line) {
            if record.filter_by_value(threshold) {
                writeln!(output_file, "{}", record.to_csv_line())?;
                records_written += 1;
            }
        }
    }

    println!("Processing complete:");
    println!("  Records processed: {}", records_processed);
    println!("  Records written: {}", records_written);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/filtered_output.csv";
    let threshold = 50.0;

    match process_csv_file(input_file, output_file, threshold) {
        Ok(_) => println!("Successfully processed CSV data"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = Record::from_csv_line("1,TestItem,75.5,true").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "TestItem");
        assert_eq!(record.value, 75.5);
        assert_eq!(record.active, true);
    }

    #[test]
    fn test_filter_logic() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 60.0,
            active: true,
        };
        assert!(record.filter_by_value(50.0));
        assert!(!record.filter_by_value(70.0));
    }

    #[test]
    fn test_inactive_record_filter() {
        let record = Record {
            id: 2,
            name: "Inactive".to_string(),
            value: 80.0,
            active: false,
        };
        assert!(!record.filter_by_value(50.0));
    }
}