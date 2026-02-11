use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    data: Vec<HashMap<String, String>>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor { data: Vec::new() }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_line) = lines.next() {
            let headers: Vec<String> = header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            for line in lines {
                let line = line?;
                let values: Vec<String> = line
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                if values.len() == headers.len() {
                    let mut row = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        row.insert(header.clone(), values[i].clone());
                    }
                    self.data.push(row);
                }
            }
        }

        Ok(())
    }

    pub fn aggregate_by_column(&self, column: &str) -> HashMap<String, usize> {
        let mut result = HashMap::new();
        for row in &self.data {
            if let Some(value) = row.get(column) {
                *result.entry(value.clone()).or_insert(0) += 1;
            }
        }
        result
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Vec<HashMap<String, String>>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        self.data
            .iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
    }

    pub fn get_unique_values(&self, column: &str) -> Vec<String> {
        let mut values = Vec::new();
        for row in &self.data {
            if let Some(value) = row.get(column) {
                if !values.contains(value) {
                    values.push(value.clone());
                }
            }
        }
        values.sort();
        values
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
        writeln!(temp_file, "Charlie,30,Paris").unwrap();

        let mut processor = CsvProcessor::new();
        processor
            .load_from_file(temp_file.path().to_str().unwrap())
            .unwrap();

        assert_eq!(processor.data.len(), 3);

        let age_distribution = processor.aggregate_by_column("age");
        assert_eq!(age_distribution.get("30"), Some(&2));
        assert_eq!(age_distribution.get("25"), Some(&1));

        let filtered = processor.filter_rows(|row| {
            row.get("age").map_or(false, |age| age == "30")
        });
        assert_eq!(filtered.len(), 2);

        let cities = processor.get_unique_values("city");
        assert_eq!(cities, vec!["London", "New York", "Paris"]);
    }
}