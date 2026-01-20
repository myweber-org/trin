
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>().unwrap_or(0.0);
            let category = parts[3].to_string();

            let record = DataRecord::new(id, name, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|record| record.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average();

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.0,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.75,CategoryA").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);

        let category_a = processor.filter_by_category("CategoryA");
        assert_eq!(category_a.len(), 2);

        let stats = processor.get_statistics();
        assert_eq!(stats.0, 10.5);
        assert_eq!(stats.1, 20.0);

        processor.clear();
        assert_eq!(processor.count_records(), 0);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_data(input_path: &str, output_path: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= threshold && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn calculate_statistics(path: &str) -> Result<(f64, f64, usize), Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut sum = 0.0;
    let mut count = 0;
    let mut max_value = f64::MIN;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.active {
            sum += record.value;
            count += 1;
            if record.value > max_value {
                max_value = record.value;
            }
        }
    }

    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    Ok((average, max_value, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let input_data = "id,name,value,active\n1,test1,10.5,true\n2,test2,5.0,false\n3,test3,15.0,true\n";
        let input_file = NamedTempFile::new().unwrap();
        std::fs::write(input_file.path(), input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        process_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            10.0
        ).unwrap();
        
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("test1"));
        assert!(!output_content.contains("test2"));
        assert!(output_content.contains("test3"));
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct UserData {
    pub username: String,
    pub email: String,
    pub age: u8,
}

impl UserData {
    pub fn new(username: String, email: String, age: u8) -> Result<Self, ValidationError> {
        if username.len() < 3 || username.len() > 20 {
            return Err(ValidationError {
                message: "Username must be between 3 and 20 characters".to_string(),
            });
        }

        if !email.contains('@') {
            return Err(ValidationError {
                message: "Email must contain '@' character".to_string(),
            });
        }

        if age < 18 || age > 120 {
            return Err(ValidationError {
                message: "Age must be between 18 and 120".to_string(),
            });
        }

        Ok(Self {
            username,
            email,
            age,
        })
    }

    pub fn normalize_username(&mut self) {
        self.username = self.username.trim().to_lowercase();
    }

    pub fn normalize_email(&mut self) {
        self.email = self.email.trim().to_lowercase();
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"username":"{}","email":"{}","age":{}}}"#,
            self.username, self.email, self.age
        )
    }
}

pub fn process_user_input(
    username: &str,
    email: &str,
    age: u8,
) -> Result<String, ValidationError> {
    let mut user_data = UserData::new(username.to_string(), email.to_string(), age)?;
    
    user_data.normalize_username();
    user_data.normalize_email();
    
    Ok(user_data.to_json())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_user_data() {
        let result = UserData::new("alice".to_string(), "alice@example.com".to_string(), 25);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_username() {
        let result = UserData::new("ab".to_string(), "test@example.com".to_string(), 25);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_email() {
        let result = UserData::new("bob".to_string(), "invalid-email".to_string(), 30);
        assert!(result.is_err());
    }

    #[test]
    fn test_normalization() {
        let mut user = UserData::new("  JOHN  ".to_string(), "JOHN@EXAMPLE.COM".to_string(), 30).unwrap();
        user.normalize_username();
        user.normalize_email();
        
        assert_eq!(user.username, "john");
        assert_eq!(user.email, "john@example.com");
    }

    #[test]
    fn test_json_output() {
        let user = UserData::new("charlie".to_string(), "charlie@test.com".to_string(), 35).unwrap();
        let json = user.to_json();
        assert!(json.contains("\"username\":\"charlie\""));
        assert!(json.contains("\"email\":\"charlie@test.com\""));
        assert!(json.contains("\"age\":35"));
    }
}