use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn parse_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

fn validate_records(records: &[Record]) -> Vec<&Record> {
    records.iter()
        .filter(|r| r.active && !r.name.is_empty())
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = parse_csv_file("data.csv")?;
    let valid_records = validate_records(&records);
    
    println!("Total records: {}", records.len());
    println!("Valid active records: {}", valid_records.len());
    
    for record in valid_records.iter().take(5) {
        println!("{:?}", record);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_valid_csv() {
        let csv_data = "id,name,value,active\n1,Test1,10.5,true\n2,Test2,-5.0,false";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = parse_csv_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            Record { id: 1, name: "Valid".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "Inactive".to_string(), value: 30.0, active: false },
        ];
        
        let valid = validate_records(&records);
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0].id, 1);
    }
}