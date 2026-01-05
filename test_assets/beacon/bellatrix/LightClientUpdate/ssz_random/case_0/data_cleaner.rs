use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;
    
    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;
    
    for result in reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| field.trim().to_string())
            .collect();
        
        writer.write_record(&cleaned_record)?;
    }
    
    writer.flush()?;
    Ok(())
}

pub fn remove_empty_rows(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;
    
    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;
    
    for result in reader.records() {
        let record = result?;
        let has_data = record.iter().any(|field| !field.trim().is_empty());
        
        if has_data {
            writer.write_record(&record)?;
        }
    }
    
    writer.flush()?;
    Ok(())
}