use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder, Trim};

#[derive(Debug, Clone)]
pub struct CleanedRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct DataCleaner {
    max_invalid_rows: usize,
    strict_mode: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            max_invalid_rows: 10,
            strict_mode: false,
        }
    }

    pub fn with_strict_mode(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    pub fn clean_csv(&self, input_path: &str, output_path: &str) -> Result<CleanStats, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        
        let output_file = File::create(output_path)?;
        let writer = BufWriter::new(output_file);
        
        let mut csv_reader = ReaderBuilder::new()
            .trim(Trim::All)
            .has_headers(true)
            .from_reader(reader);
            
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);
            
        let mut stats = CleanStats::new();
        let mut invalid_rows = 0;
        
        for result in csv_reader.records() {
            let record = match result {
                Ok(rec) => rec,
                Err(e) => {
                    stats.invalid_format += 1;
                    if self.strict_mode || invalid_rows >= self.max_invalid_rows {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Too many invalid rows: {}", e)
                        )));
                    }
                    invalid_rows += 1;
                    continue;
                }
            };
            
            let cleaned = match self.validate_and_clean(&record) {
                Some(rec) => rec,
                None => {
                    stats.invalid_content += 1;
                    continue;
                }
            };
            
            csv_writer.serialize(&cleaned)?;
            stats.processed += 1;
        }
        
        Ok(stats)
    }
    
    fn validate_and_clean(&self, record: &csv::StringRecord) -> Option<CleanedRecord> {
        if record.len() < 4 {
            return None;
        }
        
        let id = match record[0].parse::<u32>() {
            Ok(val) if val > 0 => val,
            _ => return None,
        };
        
        let name = record[1].trim().to_string();
        if name.is_empty() || name.len() > 100 {
            return None;
        }
        
        let value = match record[2].parse::<f64>() {
            Ok(val) if val >= 0.0 => val,
            _ => return None,
        };
        
        let category = record[3].trim().to_string();
        if category.is_empty() {
            return None;
        }
        
        Some(CleanedRecord {
            id,
            name,
            value,
            category,
        })
    }
}

pub struct CleanStats {
    pub processed: usize,
    pub invalid_format: usize,
    pub invalid_content: usize,
}

impl CleanStats {
    fn new() -> Self {
        CleanStats {
            processed: 0,
            invalid_format: 0,
            invalid_content: 0,
        }
    }
    
    pub fn total_processed(&self) -> usize {
        self.processed + self.invalid_format + self.invalid_content
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total_processed() == 0 {
            return 0.0;
        }
        self.processed as f64 / self.total_processed() as f64 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_clean_valid_data() {
        let input_data = "id,name,value,category\n1,Test Product,25.5,Electronics\n2,Another Item,100.0,Books";
        
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let cleaner = DataCleaner::new();
        let stats = cleaner.clean_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        assert_eq!(stats.processed, 2);
        assert_eq!(stats.success_rate(), 100.0);
    }
    
    #[test]
    fn test_filter_invalid_data() {
        let input_data = "id,name,value,category\n1,Valid,10.0,Good\nx,Invalid,-5.0,Bad\n3,,15.0,Empty";
        
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let cleaner = DataCleaner::new();
        let stats = cleaner.clean_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        assert_eq!(stats.processed, 1);
        assert_eq!(stats.invalid_content, 2);
    }
}