
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn read_and_validate(&self, file_path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(format!("Empty line at line {}", line_number).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("File contains no data".into());
        }

        Ok(records)
    }

    pub fn transform_numeric_fields(
        &self,
        records: &[Vec<String>],
        column_index: usize,
        multiplier: f64,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let mut transformed = Vec::new();
        let start_index = if self.has_headers { 1 } else { 0 };

        for (i, record) in records.iter().enumerate() {
            if i == 0 && self.has_headers {
                transformed.push(record.clone());
                continue;
            }

            if column_index >= record.len() {
                return Err(format!(
                    "Column index {} out of bounds for record at line {}",
                    column_index,
                    i + 1
                )
                .into());
            }

            let mut new_record = record.clone();
            match record[column_index].parse::<f64>() {
                Ok(value) => {
                    let transformed_value = value * multiplier;
                    new_record[column_index] = transformed_value.to_string();
                }
                Err(_) => {
                    return Err(format!(
                        "Non-numeric value in column {} at line {}",
                        column_index,
                        i + 1
                    )
                    .into());
                }
            }
            transformed.push(new_record);
        }

        Ok(transformed)
    }

    pub fn filter_records(
        &self,
        records: &[Vec<String>],
        predicate: impl Fn(&[String]) -> bool,
    ) -> Vec<Vec<String>> {
        let start_index = if self.has_headers { 1 } else { 0 };
        let mut filtered = Vec::new();

        if self.has_headers && !records.is_empty() {
            filtered.push(records[0].clone());
        }

        for record in records.iter().skip(start_index) {
            if predicate(record) {
                filtered.push(record.clone());
            }
        }

        filtered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor
            .read_and_validate(temp_file.path().to_str().unwrap())
            .unwrap();

        assert_eq!(records.len(), 4);
        assert_eq!(records[0], vec!["name", "age", "salary"]);

        let transformed = processor
            .transform_numeric_fields(&records, 2, 1.1)
            .unwrap();
        assert_eq!(transformed[1][2], "55000");
        assert_eq!(transformed[2][2], "49500");
        assert_eq!(transformed[3][2], "66000");

        let filtered = processor.filter_records(&records, |record| {
            record[1].parse::<i32>().unwrap_or(0) > 30
        });
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[1][0], "Charlie");
    }
}
use std::error::Error;
use std::fs::File;
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, name: &str, category: &str, value: f64, active: bool) -> Self {
        Self {
            id,
            name: name.to_string(),
            category: category.to_string(),
            value,
            active,
        }
    }
}

struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.active)
            .cloned()
            .collect()
    }

    fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value_record(&self) -> Option<DataRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let file = File::create(output_path)?;
        let mut wtr = WriterBuilder::new().from_writer(file);

        for record in filtered {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    fn remove_inactive_records(&mut self) {
        self.records.retain(|r| r.active);
    }
}

fn process_sample_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();

    processor.add_record(DataRecord::new(1, "ItemA", "Electronics", 299.99, true));
    processor.add_record(DataRecord::new(2, "ItemB", "Books", 24.50, true));
    processor.add_record(DataRecord::new(3, "ItemC", "Electronics", 159.75, false));
    processor.add_record(DataRecord::new(4, "ItemD", "Clothing", 45.00, true));
    processor.add_record(DataRecord::new(5, "ItemE", "Electronics", 399.99, true));

    let electronics = processor.filter_by_category("Electronics");
    println!("Found {} electronics items", electronics.len());

    let avg_value = processor.calculate_average_value();
    println!("Average value: {:.2}", avg_value);

    if let Some(max_record) = processor.find_max_value_record() {
        println!("Max value record: {:?}", max_record);
    }

    processor.remove_inactive_records();
    println!("Active records count: {}", processor.records.len());

    Ok(())
}

fn main() {
    if let Err(e) = process_sample_data() {
        eprintln!("Processing error: {}", e);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 4 {
                let record = Record {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    value: parts[2].parse()?,
                    category: parts[3].to_string(),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_records_count(&self) -> usize {
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,25.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,42.0,Books").unwrap();
        writeln!(temp_file, "3,ItemC,18.75,Electronics").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.get_records_count(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let average = processor.calculate_average();
        assert!((average - 28.75).abs() < 0.001);
        
        let max_record = processor.find_max_value();
        assert_eq!(max_record.unwrap().name, "ItemB");
    }
}