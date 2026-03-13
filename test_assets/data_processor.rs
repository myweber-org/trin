use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data(input_path: &str, output_path: &str, category_filter: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.category == category_filter && record.value > 0.0 {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn calculate_average(input_path: &str) -> Result<f64, Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let mut total = 0.0;
    let mut count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        total += record.value;
        count += 1;
    }

    if count > 0 {
        Ok(total / count as f64)
    } else {
        Ok(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_data() -> Result<(), Box<dyn Error>> {
        let input_data = "id,name,value,category\n1,test1,10.5,A\n2,test2,15.0,B\n3,test3,20.0,A";
        let input_file = NamedTempFile::new()?;
        std::fs::write(input_file.path(), input_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        process_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            "A"
        )?;
        
        let output_content = std::fs::read_to_string(output_file.path())?;
        assert!(output_content.contains("test1"));
        assert!(!output_content.contains("test2"));
        assert!(output_content.contains("test3"));
        
        Ok(())
    }

    #[test]
    fn test_calculate_average() -> Result<(), Box<dyn Error>> {
        let input_data = "id,name,value,category\n1,test1,10.0,A\n2,test2,20.0,B\n3,test3,30.0,A";
        let input_file = NamedTempFile::new()?;
        std::fs::write(input_file.path(), input_data)?;
        
        let avg = calculate_average(input_file.path().to_str().unwrap())?;
        assert_eq!(avg, 20.0);
        
        Ok(())
    }
}