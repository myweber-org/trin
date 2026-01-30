use std::error::Error;
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

    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 && self.has_headers {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn write_file<P: AsRef<Path>>(
        &self,
        path: P,
        data: &[Vec<String>],
        headers: Option<&[String]>,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;

        if let Some(headers) = headers {
            let header_line = headers.join(&self.delimiter.to_string());
            writeln!(file, "{}", header_line)?;
        }

        for record in data {
            let line = record.join(&self.delimiter.to_string());
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }

    pub fn filter_records<F>(&self, records: &[Vec<String>], predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let processor = CsvProcessor::new(',', true);
        
        let test_data = vec![
            vec!["name".to_string(), "age".to_string(), "city".to_string()],
            vec!["Alice".to_string(), "30".to_string(), "New York".to_string()],
            vec!["Bob".to_string(), "25".to_string(), "London".to_string()],
            vec!["Charlie".to_string(), "35".to_string(), "Paris".to_string()],
        ];

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        processor
            .write_file(path, &test_data[1..], Some(&test_data[0]))
            .unwrap();

        let mut file_content = String::new();
        File::open(path)
            .unwrap()
            .read_to_string(&mut file_content)
            .unwrap();

        assert!(file_content.contains("Alice,30,New York"));
        assert!(file_content.contains("name,age,city"));

        let read_data = processor.read_file(path).unwrap();
        assert_eq!(read_data.len(), 3);
        assert_eq!(read_data[0], vec!["Alice", "30", "New York"]);

        let filtered = processor.filter_records(&read_data, |record| {
            record.get(1).map_or(false, |age| age.parse::<i32>().unwrap_or(0) > 30)
        });
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["Charlie", "35", "Paris"]);

        let ages = processor.extract_column(&read_data, 1);
        assert_eq!(ages, vec!["30", "25", "35"]);
    }
}