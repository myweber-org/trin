use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn clean_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed_line = line.trim();

        if !trimmed_line.is_empty() {
            let cleaned_columns: Vec<String> = trimmed_line
                .split(',')
                .map(|col| col.trim().to_string())
                .collect();

            if cleaned_columns.iter().any(|col| !col.is_empty()) {
                writeln!(output_file, "{}", cleaned_columns.join(","))?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_clean_csv() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";

        let test_data = "  col1, col2 , col3  \n\nvalue1, value2 , value3\n  ,,  \nlast1,last2,last3  ";
        fs::write(test_input, test_data).unwrap();

        clean_csv_file(test_input, test_output).unwrap();

        let result = fs::read_to_string(test_output).unwrap();
        let expected = "col1,col2,col3\nvalue1,value2,value3\nlast1,last2,last3\n";

        assert_eq!(result, expected);

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}use std::collections::HashMap;

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

        let q1_index = (sorted_data.len() as f64 * 0.25).floor() as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75).floor() as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.thresholds.insert("lower_bound".to_string(), lower_bound);
        self.thresholds.insert("upper_bound".to_string(), upper_bound);
        self.thresholds.insert("q1".to_string(), q1);
        self.thresholds.insert("q3".to_string(), q3);

        (q1, q3, iqr, lower_bound)
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
        summary.insert("min".to_string(), self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
        summary.insert("max".to_string(), self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
        summary.insert("mean".to_string(), self.data.iter().sum::<f64>() / self.data.len() as f64);
        summary.insert("count".to_string(), self.data.len() as f64);
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

    #[test]
    fn test_summary_statistics() {
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let cleaner = DataCleaner::new(data);
        let summary = cleaner.get_summary();
        assert_eq!(summary.get("mean").unwrap(), &30.0);
        assert_eq!(summary.get("count").unwrap(), &5.0);
    }
}use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
}

impl<T> DataCleaner<T>
where
    T: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, items: Vec<T>) -> Vec<T> {
        let mut result = Vec::new();
        
        for item in items {
            if !self.seen.contains(&item) {
                self.seen.insert(item.clone());
                result.push(item);
            }
        }
        
        result
    }

    pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
        strings
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn reset(&mut self) {
        self.seen.clear();
    }
}

impl<T> Default for DataCleaner<T>
where
    T: Hash + Eq + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_integers() {
        let mut cleaner = DataCleaner::new();
        let input = vec![1, 2, 2, 3, 4, 4, 4, 5];
        let result = cleaner.deduplicate(input);
        
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_deduplicate_strings() {
        let mut cleaner = DataCleaner::new();
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        let result = cleaner.deduplicate(input);
        
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec![
            "  Hello  ".to_string(),
            "WORLD".to_string(),
            "".to_string(),
            "  Rust  ".to_string(),
        ];
        let result = DataCleaner::normalize_strings(input);
        
        assert_eq!(result, vec!["hello", "world", "rust"]);
    }

    #[test]
    fn test_reset() {
        let mut cleaner = DataCleaner::new();
        let input1 = vec![1, 2, 3];
        cleaner.deduplicate(input1);
        
        cleaner.reset();
        
        let input2 = vec![1, 2, 3];
        let result = cleaner.deduplicate(input2);
        
        assert_eq!(result, vec![1, 2, 3]);
    }
}