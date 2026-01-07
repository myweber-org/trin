
use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    seen_items: HashSet<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen_items: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, items: Vec<T>) -> Vec<T> {
        let mut unique_items = Vec::new();
        
        for item in items {
            if self.seen_items.insert(item.clone()) {
                unique_items.push(item);
            }
        }
        
        unique_items
    }

    pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
        strings
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn reset(&mut self) {
        self.seen_items.clear();
    }

    pub fn get_unique_count(&self) -> usize {
        self.seen_items.len()
    }
}

pub fn remove_outliers(values: &[f64], threshold: f64) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }

    let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
    let variance: f64 = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    let std_dev = variance.sqrt();

    values.iter()
        .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        let data = vec![1, 2, 2, 3, 3, 3, 4];
        let result = cleaner.deduplicate(data);
        assert_eq!(result, vec![1, 2, 3, 4]);
        assert_eq!(cleaner.get_unique_count(), 4);
    }

    #[test]
    fn test_normalize_strings() {
        let strings = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "   ".to_string(),
            "RUST".to_string(),
        ];
        let result = DataCleaner::normalize_strings(strings);
        assert_eq!(result, vec!["hello", "world", "rust"]);
    }

    #[test]
    fn test_remove_outliers() {
        let values = vec![1.0, 2.0, 2.5, 3.0, 100.0];
        let result = remove_outliers(&values, 2.0);
        assert!(result.contains(&1.0));
        assert!(result.contains(&2.0));
        assert!(result.contains(&2.5));
        assert!(result.contains(&3.0));
        assert!(!result.contains(&100.0));
    }
}