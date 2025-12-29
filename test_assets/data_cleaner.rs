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

    pub fn calculate_iqr_thresholds(&mut self) -> (f64, f64) {
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25) as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75) as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.thresholds.insert("lower".to_string(), lower_bound);
        self.thresholds.insert("upper".to_string(), upper_bound);

        (lower_bound, upper_bound)
    }

    pub fn remove_outliers(&self) -> Vec<f64> {
        let lower = self.thresholds.get("lower").unwrap_or(&f64::MIN);
        let upper = self.thresholds.get("upper").unwrap_or(&f64::MAX);

        self.data
            .iter()
            .filter(|&&value| value >= *lower && value <= *upper)
            .cloned()
            .collect()
    }

    pub fn get_summary_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let mean = sum / self.data.len() as f64;
        
        let variance: f64 = self.data.iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / self.data.len() as f64;
        
        let std_dev = variance.sqrt();

        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), *self.data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        stats.insert("max".to_string(), *self.data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        stats.insert("count".to_string(), self.data.len() as f64);

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
        
        cleaner.calculate_iqr_thresholds();
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_empty_data() {
        let data = vec![];
        let cleaner = DataCleaner::new(data);
        
        let stats = cleaner.get_summary_stats();
        assert!(stats.is_empty());
    }
}