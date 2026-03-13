
use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    thresholds: HashMap<String, f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner {
            data,
            thresholds: HashMap::new(),
        }
    }

    pub fn calculate_iqr(&mut self) -> (f64, f64, f64, f64) {
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25) as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75) as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.thresholds.insert("lower_bound".to_string(), lower_bound);
        self.thresholds.insert("upper_bound".to_string(), upper_bound);
        self.thresholds.insert("q1".to_string(), q1);
        self.thresholds.insert("q3".to_string(), q3);

        (q1, q3, lower_bound, upper_bound)
    }

    pub fn remove_outliers(&self) -> Vec<f64> {
        let lower_bound = self.thresholds.get("lower_bound").unwrap_or(&f64::MIN);
        let upper_bound = self.thresholds.get("upper_bound").unwrap_or(&f64::MAX);

        self.data
            .iter()
            .filter(|&&value| value >= *lower_bound && value <= *upper_bound)
            .cloned()
            .collect()
    }

    pub fn get_summary(&self) -> HashMap<String, f64> {
        let mut summary = HashMap::new();
        summary.insert("min".to_string(), *self.data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0));
        summary.insert("max".to_string(), *self.data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0));
        summary.insert("mean".to_string(), self.data.iter().sum::<f64>() / self.data.len() as f64);
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data);
        cleaner.calculate_iqr();
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
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
    category: String,
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_uppercase();
        
        if record.value < 0.0 {
            record.value = 0.0;
        }
        
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && 
    record.id > 0 && 
    record.category.len() <= 10
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";
    
    match clean_csv(input_file, output_file) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error during data cleaning: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "CATEGORY".to_string(),
        };
        
        let invalid_record = Record {
            id: 0,
            name: "".to_string(),
            value: -5.0,
            category: "TOOLONGCATEGORYNAME".to_string(),
        };
        
        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn clean_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(Path::new(output_path))?;

    for line in reader.lines() {
        let original_line = line?;
        let trimmed_line = original_line.trim();

        if !trimmed_line.is_empty() {
            let cleaned_columns: Vec<String> = trimmed_line
                .split(',')
                .map(|col| col.trim().to_string())
                .collect();

            writeln!(output_file, "{}", cleaned_columns.join(","))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_clean_csv() {
        let test_input = "data, value , extra\n\n,test, spaces  \nlast,row,";
        let temp_input = "test_input.csv";
        let temp_output = "test_output.csv";

        let mut input_file = File::create(temp_input).unwrap();
        write!(input_file, "{}", test_input).unwrap();

        clean_csv_file(temp_input, temp_output).unwrap();

        let mut output_file = File::open(temp_output).unwrap();
        let mut contents = String::new();
        output_file.read_to_string(&mut contents).unwrap();

        let expected = "data,value,extra\ntest,spaces\nlast,row,\n";
        assert_eq!(contents, expected);

        std::fs::remove_file(temp_input).unwrap();
        std::fs::remove_file(temp_output).unwrap();
    }
}
use std::collections::HashMap;

pub struct DataCleaner {
    pub strict_mode: bool,
}

impl DataCleaner {
    pub fn new(strict: bool) -> Self {
        DataCleaner {
            strict_mode: strict,
        }
    }

    pub fn clean_string_vector(&self, data: Vec<Option<String>>) -> Vec<String> {
        data.into_iter()
            .filter_map(|item| {
                match item {
                    Some(mut s) => {
                        s = s.trim().to_string();
                        if s.is_empty() && self.strict_mode {
                            None
                        } else if s.is_empty() {
                            Some("N/A".to_string())
                        } else {
                            Some(s)
                        }
                    }
                    None if self.strict_mode => None,
                    None => Some("N/A".to_string()),
                }
            })
            .collect()
    }

    pub fn clean_hashmap(
        &self,
        data: HashMap<String, Option<f64>>,
    ) -> HashMap<String, f64> {
        data.into_iter()
            .filter_map(|(key, value)| {
                value.and_then(|v| {
                    if v.is_finite() {
                        Some((key, v))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub fn calculate_statistics(&self, numbers: &[f64]) -> Option<(f64, f64)> {
        if numbers.is_empty() {
            return None;
        }

        let sum: f64 = numbers.iter().sum();
        let mean = sum / numbers.len() as f64;
        let variance: f64 = numbers
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / numbers.len() as f64;

        Some((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string_vector_strict() {
        let cleaner = DataCleaner::new(true);
        let data = vec![
            Some("  hello  ".to_string()),
            Some("".to_string()),
            None,
            Some("world".to_string()),
        ];
        let result = cleaner.clean_string_vector(data);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_clean_hashmap() {
        let cleaner = DataCleaner::new(false);
        let mut data = HashMap::new();
        data.insert("a".to_string(), Some(1.5));
        data.insert("b".to_string(), None);
        data.insert("c".to_string(), Some(f64::NAN));

        let result = cleaner.clean_hashmap(data);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("a"), Some(&1.5));
    }
}