use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn filter_and_transform(input_path: &str, output_path: &str, min_age: u8) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.age >= min_age && record.active {
            let transformed = Record {
                id: record.id,
                name: record.name.to_uppercase(),
                age: record.age,
                active: record.active,
            };
            writer.serialize(transformed)?;
        }
    }
    
    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let minimum_age = 25;
    
    filter_and_transform(input_file, output_file, minimum_age)?;
    
    println!("Processing completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_and_transform() -> Result<(), Box<dyn Error>> {
        let mut input_temp = NamedTempFile::new()?;
        writeln!(input_temp, "id,name,age,active")?;
        writeln!(input_temp, "1,alice,30,true")?;
        writeln!(input_temp, "2,bob,20,true")?;
        writeln!(input_temp, "3,charlie,40,false")?;
        
        let output_temp = NamedTempFile::new()?;
        
        filter_and_transform(
            input_temp.path().to_str().unwrap(),
            output_temp.path().to_str().unwrap(),
            25
        )?;
        
        let mut reader = Reader::from_reader(File::open(output_temp.path())?);
        let records: Vec<Record> = reader.deserialize().collect::<Result<_, _>>()?;
        
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].name, "ALICE");
        
        Ok(())
    }
}