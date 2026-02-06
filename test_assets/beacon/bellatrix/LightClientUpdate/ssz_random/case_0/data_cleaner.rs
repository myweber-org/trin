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
}use std::collections::HashSet;

pub fn clean_and_sort_data(input: Vec<String>) -> Vec<String> {
    let unique_items: HashSet<String> = input.into_iter().collect();
    let mut sorted_items: Vec<String> = unique_items.into_iter().collect();
    sorted_items.sort();
    sorted_items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_sort() {
        let input = vec![
            "banana".to_string(),
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
            "apple".to_string(),
        ];
        let result = clean_and_sort_data(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }
}