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
    fn from_line(line: &str) -> Result<Self, Box<dyn Error>> {
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

fn process_csv_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        match Record::from_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Skipping line {}: {}", index + 1, e),
        }
    }

    Ok(records)
}

fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut aggregates: HashMap<String, f64> = HashMap::new();

    for record in records {
        *aggregates.entry(record.category.clone()).or_insert(0.0) += record.value;
    }

    let mut result: Vec<(String, f64)> = aggregates.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = process_csv_file("data.csv")?;
    
    println!("Loaded {} records", records.len());
    
    let aggregates = aggregate_by_category(&records);
    
    println!("Aggregated values by category:");
    for (category, total) in aggregates {
        println!("{}: {:.2}", category, total);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_parsing() {
        let line = "1,ProductA,25.5,Electronics";
        let record = Record::from_line(line).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "ProductA");
        assert_eq!(record.value, 25.5);
        assert_eq!(record.category, "Electronics");
    }

    #[test]
    fn test_aggregation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Y".to_string() },
            Record { id: 3, name: "C".to_string(), value: 15.0, category: "X".to_string() },
        ];

        let aggregates = aggregate_by_category(&records);
        
        assert_eq!(aggregates.len(), 2);
        assert_eq!(aggregates[0], ("X".to_string(), 25.0));
        assert_eq!(aggregates[1], ("Y".to_string(), 20.0));
    }

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.3,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.7,CategoryA").unwrap();

        let records = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].category, "CategoryA");
        assert_eq!(records[1].value, 20.3);
    }
}