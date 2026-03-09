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
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvParser {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            records.push(record);
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Vec<Vec<String>> {
        content
            .lines()
            .enumerate()
            .filter(|(line_num, line)| {
                if *line_num == 0 && self.has_header {
                    false
                } else {
                    !line.trim().is_empty()
                }
            })
            .map(|(_, line)| {
                line.split(self.delimiter)
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .collect()
    }
}

pub fn calculate_column_average(records: &[Vec<String>], column_index: usize) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0;

    for record in records {
        if column_index < record.len() {
            if let Ok(value) = record[column_index].parse::<f64>() {
                sum += value;
                count += 1;
            }
        }
    }

    if count > 0 {
        Some(sum / count as f64)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let parser = CsvParser::new(',', true);
        let csv_data = "name,age,score\nAlice,30,95.5\nBob,25,87.0\nCharlie,35,91.2";
        
        let records = parser.parse_string(csv_data);
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "95.5"]);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "30.0".to_string()],
            vec!["12.0".to_string(), "25.0".to_string()],
        ];
        
        let avg = calculate_column_average(&records, 0);
        assert!(avg.is_some());
        assert!((avg.unwrap() - 12.666666666666666).abs() < 0.0001);
    }

    #[test]
    fn test_empty_records() {
        let records: Vec<Vec<String>> = vec![];
        let avg = calculate_column_average(&records, 0);
        assert!(avg.is_none());
    }
}