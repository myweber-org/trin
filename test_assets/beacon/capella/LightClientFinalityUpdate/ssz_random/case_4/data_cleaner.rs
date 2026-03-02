use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct DataCleaner {
    input_path: String,
    output_path: String,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    pub fn clean_csv(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        
        let output_file = File::create(&self.output_path)?;
        let mut writer = std::io::BufWriter::new(output_file);

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 {
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;
                continue;
            }

            let cleaned_line = self.process_line(&line);
            if !cleaned_line.is_empty() {
                writer.write_all(cleaned_line.as_bytes())?;
                writer.write_all(b"\n")?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    fn process_line(&self, line: &str) -> String {
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() < 3 {
            return String::new();
        }

        let mut cleaned_parts = Vec::new();
        
        for part in parts {
            let trimmed = part.trim();
            if !trimmed.is_empty() && trimmed != "null" && trimmed != "NULL" {
                cleaned_parts.push(trimmed);
            } else {
                cleaned_parts.push("");
            }
        }

        cleaned_parts.join(",")
    }

    pub fn validate_file(&self) -> bool {
        Path::new(&self.input_path).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_data_cleaner() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let content = "id,name,value\n1,John,100\n2,,200\n3,Alice,null\n4,Bob,\n";
        fs::write(test_input, content).unwrap();

        let cleaner = DataCleaner::new(test_input, test_output);
        assert!(cleaner.validate_file());
        
        let result = cleaner.clean_csv();
        assert!(result.is_ok());

        let cleaned_content = fs::read_to_string(test_output).unwrap();
        assert!(cleaned_content.contains("1,John,100"));
        assert!(!cleaned_content.contains("null"));

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}use std::collections::HashMap;

pub struct DataCleaner {
    threshold: f64,
}

impl DataCleaner {
    pub fn new(threshold: f64) -> Self {
        DataCleaner { threshold }
    }

    pub fn remove_outliers_iqr(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < 4 {
            return data.to_vec();
        }

        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25).floor() as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75).floor() as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - self.threshold * iqr;
        let upper_bound = q3 + self.threshold * iqr;

        data.iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .cloned()
            .collect()
    }

    pub fn analyze_dataset(&self, data: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if data.is_empty() {
            return stats;
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        let std_dev = variance.sqrt();

        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        stats.insert("max".to_string(), *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        stats.insert("count".to_string(), data.len() as f64);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let cleaner = DataCleaner::new(1.5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let cleaned = cleaner.remove_outliers_iqr(&data);
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_statistics() {
        let cleaner = DataCleaner::new(1.5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = cleaner.analyze_dataset(&data);
        
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("count").unwrap(), &5.0);
    }
}