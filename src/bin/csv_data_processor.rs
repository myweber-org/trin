
use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn process_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut wtr = Writer::from_path(output_path)?;

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= min_value && record.active {
            wtr.serialize(&record)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

fn generate_sample_csv(path: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;
    let records = vec![
        Record { id: 1, name: String::from("Alpha"), value: 42.5, active: true },
        Record { id: 2, name: String::from("Beta"), value: 18.3, active: false },
        Record { id: 3, name: String::from("Gamma"), value: 75.1, active: true },
        Record { id: 4, name: String::from("Delta"), value: 9.7, active: true },
    ];

    for record in records {
        wtr.serialize(&record)?;
    }
    wtr.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "input_data.csv";
    let output_file = "filtered_data.csv";
    let threshold = 20.0;

    generate_sample_csv(input_file)?;
    process_csv(input_file, output_file, threshold)?;

    println!("Processing completed. Filtered data saved to {}", output_file);
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(Record {
                id,
                name,
                value,
                category,
            });
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    sum / records.len() as f64
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_records() -> Vec<Record> {
        vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                value: 10.5,
                category: "Electronics".to_string(),
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                value: 25.0,
                category: "Books".to_string(),
            },
            Record {
                id: 3,
                name: "Item C".to_string(),
                value: 15.75,
                category: "Electronics".to_string(),
            },
        ]
    }

    #[test]
    fn test_filter_by_category() {
        let records = create_test_records();
        let filtered = filter_by_category(&records, "Electronics");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let records = create_test_records();
        let avg = calculate_average(&records);
        assert!((avg - 17.0833).abs() < 0.001);
    }

    #[test]
    fn test_find_max_value() {
        let records = create_test_records();
        let max_record = find_max_value(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 25.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn read_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(CsvRecord {
                id,
                name,
                value,
                category,
            });
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<&CsvRecord> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter().map(|record| record.value).sum()
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "id,name,value,category\n1,ItemA,25.5,Electronics\n2,ItemB,42.8,Books\n3,ItemC,18.3,Electronics"
        )
        .unwrap();
        temp_file
    }

    #[test]
    fn test_read_csv_file() {
        let temp_file = create_test_csv();
        let records = read_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 42.8);
    }

    #[test]
    fn test_filter_by_category() {
        let temp_file = create_test_csv();
        let records = read_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        let filtered = filter_by_category(&records, "Electronics");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "Electronics"));
    }

    #[test]
    fn test_calculate_total_value() {
        let temp_file = create_test_csv();
        let records = read_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        let total = calculate_total_value(&records);
        assert!((total - 86.6).abs() < 0.001);
    }

    #[test]
    fn test_find_max_value_record() {
        let temp_file = create_test_csv();
        let records = read_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.name, "ItemB");
    }
}