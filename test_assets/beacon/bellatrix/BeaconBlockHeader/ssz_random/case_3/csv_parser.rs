use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            delimiter: ',',
            has_header: true,
        }
    }

    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn has_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            if let Some((_, line)) = lines.next() {
                let line = line?;
                println!("Header: {}", line);
            }
        }

        for (line_num, line) in lines {
            let line = line?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if record.iter().any(|field| field.is_empty()) {
                eprintln!("Warning: Empty field detected at line {}", line_num + 1);
            }

            records.push(record);
        }

        Ok(records)
    }

    pub fn parse_string(&self, data: &str) -> Vec<Vec<String>> {
        data.lines()
            .skip(if self.has_header { 1 } else { 0 })
            .map(|line| {
                line.split(self.delimiter)
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .collect()
    }
}

pub fn summarize_records(records: &[Vec<String>]) {
    if records.is_empty() {
        println!("No records found");
        return;
    }

    println!("Total records: {}", records.len());
    println!("Fields per record: {}", records[0].len());

    for (i, record) in records.iter().enumerate().take(3) {
        println!("Record {}: {:?}", i + 1, record);
    }

    if records.len() > 3 {
        println!("... and {} more records", records.len() - 3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let data = "name,age,city\nJohn,30,NYC\nJane,25,London";
        let parser = CsvParser::new();
        let records = parser.parse_string(data);

        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "NYC"]);
        assert_eq!(records[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let data = "name|age|city\nJohn|30|NYC";
        let parser = CsvParser::new().delimiter('|');
        let records = parser.parse_string(data);

        assert_eq!(records[0], vec!["John", "30", "NYC"]);
    }

    #[test]
    fn test_no_header() {
        let data = "John,30,NYC\nJane,25,London";
        let parser = CsvParser::new().has_header(false);
        let records = parser.parse_string(data);

        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "NYC"]);
    }
}