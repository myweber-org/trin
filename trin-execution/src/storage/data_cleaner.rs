
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    email: String,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
            && self.age > 0
            && self.age < 120
            && self.email.contains('@')
    }

    fn sanitize(&mut self) {
        self.name = self.name.trim().to_string();
        self.email = self.email.trim().to_lowercase();
    }
}

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let mut valid_count = 0;

    for result in reader.deserialize() {
        let mut record: Record = result?;
        record.sanitize();

        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        }
    }

    writer.flush()?;
    Ok(valid_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "John Doe".to_string(),
            age: 30,
            email: "john@example.com".to_string(),
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "   ".to_string(),
            age: 0,
            email: "invalid-email".to_string(),
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_clean_csv_data() -> Result<(), Box<dyn Error>> {
        let input_data = "id,name,age,email\n1,John Doe,30,john@example.com\n2,  Alice  ,25,ALICE@EXAMPLE.COM\n3,,0,invalid\n";
        
        let input_file = NamedTempFile::new()?;
        std::fs::write(&input_file, input_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        let valid_count = clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        )?;
        
        assert_eq!(valid_count, 2);
        
        let output_content = std::fs::read_to_string(output_file.path())?;
        assert!(output_content.contains("john@example.com"));
        assert!(output_content.contains("alice@example.com"));
        assert!(!output_content.contains("invalid"));
        
        Ok(())
    }
}