
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }

            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let active = parts[3].parse::<bool>()?;

            let record = Record::new(id, name, value, active);
            record.validate()?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records.iter().find(|record| record.id == target_id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record::new(2, "".to_string(), -50.0, false);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,73.2,false").unwrap();
        writeln!(temp_file, "3,Charlie,15.8,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.filter_active().len(), 2);
        assert_eq!(processor.calculate_total(), 131.5);
        assert!(processor.find_by_id(2).is_some());
        assert!(processor.find_by_id(99).is_none());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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

    pub fn filter_columns<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        selected_columns: &[usize],
    ) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        let mut lines = reader.lines();
        
        if self.has_headers {
            if let Some(header_line) = lines.next() {
                let headers: Vec<String> = header_line?
                    .split(self.delimiter)
                    .map(String::from)
                    .collect();
                
                let filtered_headers: Vec<String> = selected_columns
                    .iter()
                    .filter_map(|&idx| headers.get(idx).cloned())
                    .collect();
                
                writeln!(output_file, "{}", filtered_headers.join(&self.delimiter.to_string()))?;
            }
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<&str> = line.split(self.delimiter).collect();
            
            let filtered_fields: Vec<String> = selected_columns
                .iter()
                .filter_map(|&idx| fields.get(idx).map(|&s| s.to_string()))
                .collect();
            
            writeln!(output_file, "{}", filtered_fields.join(&self.delimiter.to_string()))?;
        }

        Ok(())
    }

    pub fn count_rows<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let total_lines = reader.lines().count();
        
        let adjusted_count = if self.has_headers && total_lines > 0 {
            total_lines - 1
        } else {
            total_lines
        };
        
        Ok(adjusted_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_columns() {
        let input_content = "name,age,city,country\nAlice,30,London,UK\nBob,25,Paris,FR";
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(',', true);
        processor.filter_columns(
            input_file.path(),
            output_file.path(),
            &[0, 2],
        ).unwrap();
        
        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        assert_eq!(output_content, "name,city\nAlice,London\nBob,Paris\n");
    }

    #[test]
    fn test_count_rows() {
        let content = "header1,header2\nvalue1,value2\nvalue3,value4\nvalue5,value6";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", content).unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let count = processor.count_rows(temp_file.path()).unwrap();
        
        assert_eq!(count, 3);
    }
}