use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid number of fields".into());
        }

        let id = parts[0].parse()?;
        let name = parts[1].to_string();
        let value = parts[2].parse()?;
        let active = parts[3].parse()?;

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }

    fn to_csv_line(&self) -> String {
        format!("{},{},{},{}", self.id, self.name, self.value, self.active)
    }
}

fn process_csv_file(input_path: &Path, output_path: &Path, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    let mut line_count = 0;
    let mut filtered_count = 0;

    for line_result in reader.lines() {
        line_count += 1;
        let line = line_result?;

        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        match Record::from_csv_line(&line) {
            Ok(record) => {
                if record.value >= min_value && record.active {
                    writeln!(output_file, "{}", record.to_csv_line())?;
                    filtered_count += 1;
                }
            }
            Err(e) => eprintln!("Warning: Failed to parse line {}: {}", line_count, e),
        }
    }

    println!("Processed {} lines, filtered {} records", line_count, filtered_count);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = Path::new("data/input.csv");
    let output_path = Path::new("data/filtered.csv");
    let min_value = 100.0;

    if !input_path.exists() {
        return Err("Input file does not exist".into());
    }

    process_csv_file(input_path, output_path, min_value)?;
    println!("Filtered data written to {:?}", output_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_parsing() {
        let line = "42,Test Item,150.5,true";
        let record = Record::from_csv_line(line).unwrap();
        
        assert_eq!(record.id, 42);
        assert_eq!(record.name, "Test Item");
        assert_eq!(record.value, 150.5);
        assert_eq!(record.active, true);
    }

    #[test]
    fn test_invalid_record() {
        let line = "42,Test Item";
        let result = Record::from_csv_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut input_file = NamedTempFile::new()?;
        writeln!(input_file, "1,Item A,50.0,true")?;
        writeln!(input_file, "2,Item B,150.0,true")?;
        writeln!(input_file, "3,Item C,200.0,false")?;
        writeln!(input_file, "# This is a comment")?;
        writeln!(input_file, "")?;

        let output_file = NamedTempFile::new()?;
        
        process_csv_file(input_file.path(), output_file.path(), 100.0)?;
        
        let output_content = std::fs::read_to_string(output_file.path())?;
        assert!(output_content.contains("Item B"));
        assert!(!output_content.contains("Item A"));
        assert!(!output_content.contains("Item C"));
        
        Ok(())
    }
}