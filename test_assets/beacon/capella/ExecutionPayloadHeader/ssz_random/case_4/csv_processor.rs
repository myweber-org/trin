use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub fn parse_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line_content.split(',').collect();
        
        if fields.len() != 4 {
            return Err(format!("Invalid field count at line {}", line_number).into());
        }

        let id = fields[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(format!("Empty name field at line {}", line_number).into());
        }

        let value = fields[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(format!("Invalid boolean value at line {}", line_number).into()),
        };

        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }

    Ok(records)
}

pub fn validate_records(records: &[CsvRecord]) -> Result<(), Box<dyn Error>> {
    let mut seen_ids = std::collections::HashSet::new();
    
    for record in records {
        if seen_ids.contains(&record.id) {
            return Err(format!("Duplicate ID found: {}", record.id).into());
        }
        seen_ids.insert(record.id);
        
        if record.value < 0.0 {
            return Err(format!("Negative value for ID {}: {}", record.id, record.value).into());
        }
        
        if record.name.len() > 100 {
            return Err(format!("Name too long for ID {}: {} characters", record.id, record.name.len()).into());
        }
    }
    
    Ok(())
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, usize) {
    let active_records: Vec<&CsvRecord> = records.iter()
        .filter(|r| r.active)
        .collect();
    
    if active_records.is_empty() {
        return (0.0, 0.0, 0);
    }
    
    let sum: f64 = active_records.iter().map(|r| r.value).sum();
    let count = active_records.len();
    let average = sum / count as f64;
    
    let variance: f64 = active_records.iter()
        .map(|r| (r.value - average).powi(2))
        .sum::<f64>() / count as f64;
    
    (average, variance, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,42.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,17.3,false").unwrap();
        writeln!(temp_file, "3,ItemC,89.1,yes").unwrap();
        
        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 17.3);
        assert!(records[2].active);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            CsvRecord { id: 1, name: "Test".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
        ];
        
        assert!(validate_records(&records).is_ok());
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: true },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: false },
        ];
        
        let (avg, var, count) = calculate_statistics(&records);
        assert_eq!(count, 2);
        assert_eq!(avg, 15.0);
        assert_eq!(var, 25.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    pub fn filter_by_column_value(&self, column_name: &str, target_value: &str) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        
        let output_file = File::create(&self.output_path)?;
        let writer = BufWriter::new(output_file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);
        
        let headers = csv_reader.headers()?.clone();
        csv_writer.write_record(&headers)?;
        
        let column_index = headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        for result in csv_reader.records() {
            let record = result?;
            if record.get(column_index) == Some(target_value) {
                csv_writer.write_record(&record)?;
            }
        }
        
        csv_writer.flush()?;
        Ok(())
    }
    
    pub fn transform_column<F>(&self, column_name: &str, transform_fn: F) -> Result<(), Box<dyn Error>>
    where
        F: Fn(&str) -> String,
    {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        
        let output_file = File::create(&self.output_path)?;
        let writer = BufWriter::new(output_file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);
        
        let headers = csv_reader.headers()?.clone();
        csv_writer.write_record(&headers)?;
        
        let column_index = headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        for result in csv_reader.records() {
            let mut record = result?.clone();
            if let Some(value) = record.get(column_index) {
                let transformed = transform_fn(value);
                record[column_index] = transformed.into();
            }
            csv_writer.write_record(&record)?;
        }
        
        csv_writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_filter_by_column_value() {
        let input_data = "name,age,city\nAlice,30,London\nBob,25,Paris\nCharlie,35,London";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(&input_file, input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        processor.filter_by_column_value("city", "London").unwrap();
        
        let output = fs::read_to_string(output_file).unwrap();
        let expected = "name,age,city\nAlice,30,London\nCharlie,35,London\n";
        assert_eq!(output, expected);
    }
    
    #[test]
    fn test_transform_column() {
        let input_data = "name,score\nAlice,85\nBob,92";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(&input_file, input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        processor.transform_column("score", |s| format!("Grade: {}", s)).unwrap();
        
        let output = fs::read_to_string(output_file).unwrap();
        let expected = "name,score\nAlice,Grade: 85\nBob,Grade: 92\n";
        assert_eq!(output, expected);
    }
}