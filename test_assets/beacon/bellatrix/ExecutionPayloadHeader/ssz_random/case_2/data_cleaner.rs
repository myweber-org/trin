
use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    cleaned_data: Vec<f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner {
            data: data.clone(),
            cleaned_data: Vec::new(),
        }
    }

    pub fn clean_with_iqr(&mut self) -> &Vec<f64> {
        if self.data.is_empty() {
            return &self.cleaned_data;
        }

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = Self::calculate_quartile(&sorted_data, 0.25);
        let q3 = Self::calculate_quartile(&sorted_data, 0.75);
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.cleaned_data = self.data
            .iter()
            .filter(|&&value| value >= lower_bound && value <= upper_bound)
            .cloned()
            .collect();

        &self.cleaned_data
    }

    pub fn get_summary(&self) -> HashMap<String, f64> {
        let mut summary = HashMap::new();
        
        if !self.cleaned_data.is_empty() {
            let sum: f64 = self.cleaned_data.iter().sum();
            let count = self.cleaned_data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = self.cleaned_data
                .iter()
                .map(|&value| (value - mean).powi(2))
                .sum::<f64>() / count;
            
            summary.insert("original_count".to_string(), self.data.len() as f64);
            summary.insert("cleaned_count".to_string(), self.cleaned_data.len() as f64);
            summary.insert("mean".to_string(), mean);
            summary.insert("variance".to_string(), variance);
            summary.insert("std_dev".to_string(), variance.sqrt());
        }
        
        summary
    }

    fn calculate_quartile(sorted_data: &[f64], percentile: f64) -> f64 {
        let n = sorted_data.len();
        let index = percentile * (n - 1) as f64;
        
        let lower_index = index.floor() as usize;
        let upper_index = index.ceil() as usize;
        
        if lower_index == upper_index {
            sorted_data[lower_index]
        } else {
            let weight = index - lower_index as f64;
            sorted_data[lower_index] * (1.0 - weight) + sorted_data[upper_index] * weight
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_with_iqr() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data);
        let cleaned = cleaner.clean_with_iqr();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_empty_data() {
        let data = vec![];
        let mut cleaner = DataCleaner::new(data);
        let cleaned = cleaner.clean_with_iqr();
        
        assert!(cleaned.is_empty());
    }
}