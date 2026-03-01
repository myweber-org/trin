
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

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        let mut stats = HashMap::new();
        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);

        stats
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
    fn test_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data);
        let stats = cleaner.get_statistics();

        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("count").unwrap(), &5.0);
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn deduplicate(&mut self) -> usize {
        let mut seen = HashSet::new();
        let mut deduped = Vec::new();
        
        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                deduped.push(record);
            }
        }
        
        let removed = self.records.capacity() - deduped.len();
        self.records = deduped;
        removed
    }

    pub fn validate_records(&self) -> (usize, usize) {
        let mut valid_count = 0;
        
        for record in &self.records {
            if !record.trim().is_empty() && record.len() <= 1000 {
                valid_count += 1;
            }
        }
        
        (valid_count, self.records.len() - valid_count)
    }

    pub fn get_records(&self) -> &[String] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());
        
        let removed = cleaner.deduplicate();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.get_records().len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("x".repeat(1001));
        
        let (valid, invalid) = cleaner.validate_records();
        assert_eq!(valid, 1);
        assert_eq!(invalid, 2);
    }
}