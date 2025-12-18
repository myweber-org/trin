use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn get_column(&self, column_name: &str) -> Option<Vec<&str>> {
        let index = self.headers.iter()
            .position(|h| h == column_name)?;
        
        Some(self.records.iter()
            .filter_map(|record| record.get(index))
            .map(|s| s.as_str())
            .collect())
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<&Vec<String>>
    where
        F: Fn(&Vec<String>) -> bool,
    {
        self.records.iter()
            .filter(|record| predicate(record))
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn header_count(&self) -> usize {
        self.headers.len()
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap())
            .expect("Failed to read CSV");

        assert_eq!(processor.header_count(), 3);
        assert_eq!(processor.record_count(), 3);

        let ages = processor.get_column("age").unwrap();
        assert_eq!(ages, vec!["30", "25", "35"]);

        let young_people = processor.filter_records(|record| {
            record.get(1).and_then(|age| age.parse::<i32>().ok())
                .map_or(false, |age| age < 30)
        });
        assert_eq!(young_people.len(), 1);
        assert_eq!(young_people[0][0], "Bob");
    }
}