use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

#[derive(Debug)]
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
        let active = parts[3].parse().unwrap_or(false);

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

    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

fn process_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    let mut processed_count = 0;
    let mut skipped_count = 0;

    for (index, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        
        if index == 0 {
            writeln!(output_file, "{}", line)?;
            continue;
        }

        if let Some(mut record) = Record::from_csv_line(&line) {
            if record.is_valid() && record.value >= min_value {
                record.name = record.name.to_uppercase();
                writeln!(output_file, "{}", record.to_csv_line())?;
                processed_count += 1;
            } else {
                skipped_count += 1;
            }
        } else {
            eprintln!("Warning: Invalid CSV format at line {}", index + 1);
            skipped_count += 1;
        }
    }

    println!("Processing complete:");
    println!("  Records processed: {}", processed_count);
    println!("  Records skipped: {}", skipped_count);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let filter_threshold = 100.0;

    match process_csv(input_file, output_file, filter_threshold) {
        Ok(_) => println!("CSV processing successful"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_record_parsing() {
        let valid_line = "42,test item,150.5,true";
        let record = Record::from_csv_line(valid_line).unwrap();
        
        assert_eq!(record.id, 42);
        assert_eq!(record.name, "test item");
        assert_eq!(record.value, 150.5);
        assert_eq!(record.active, true);
    }

    #[test]
    fn test_invalid_record() {
        let invalid_line = "not_a_number,item,value,true";
        assert!(Record::from_csv_line(invalid_line).is_none());
    }

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "test".to_string(),
            value: 50.0,
            active: true,
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            active: false,
        };
        assert!(!invalid_record.is_valid());
    }
}