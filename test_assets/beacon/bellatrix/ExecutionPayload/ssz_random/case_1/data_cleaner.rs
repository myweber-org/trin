
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);
    
    let mut cleaned_count = 0;
    let mut skipped_count = 0;
    
    for result in csv_reader.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping malformed record: {}", e);
                skipped_count += 1;
                continue;
            }
        };
        
        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            value: if record.value.is_finite() {
                record.value
            } else {
                0.0
            },
            category: record.category.to_uppercase(),
        };
        
        csv_writer.serialize(&cleaned_record)?;
        cleaned_count += 1;
    }
    
    csv_writer.flush()?;
    
    println!("Data cleaning completed:");
    println!("  Cleaned records: {}", cleaned_count);
    println!("  Skipped records: {}", skipped_count);
    
    Ok(())
}

fn validate_file_path(path: &str) -> Result<(), io::Error> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {}", path)
        ));
    }
    
    if !path_obj.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Path is not a file: {}", path)
        ));
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "input_data.csv";
    let output_file = "cleaned_data.csv";
    
    match validate_file_path(input_file) {
        Ok(_) => {
            println!("Processing file: {}", input_file);
            clean_csv_data(input_file, output_file)?;
            println!("Output saved to: {}", output_file);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(Box::new(e));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_clean_csv_data() {
        let mut input_file = NamedTempFile::new().unwrap();
        let input_content = "id,name,value,category\n1, test ,3.14,science\n2,data,NaN,technology\n";
        write!(input_file, "{}", input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        assert!(result.is_ok());
        
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("SCIENCE"));
        assert!(output_content.contains("TECHNOLOGY"));
    }
    
    #[test]
    fn test_validate_file_path() {
        let temp_file = NamedTempFile::new().unwrap();
        assert!(validate_file_path(temp_file.path().to_str().unwrap()).is_ok());
        
        let result = validate_file_path("non_existent_file.csv");
        assert!(result.is_err());
    }
}