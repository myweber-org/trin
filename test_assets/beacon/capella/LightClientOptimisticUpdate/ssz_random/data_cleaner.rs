use std::collections::HashSet;

pub fn remove_duplicates<T: Eq + std::hash::Hash + Clone>(data: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in data {
        if seen.insert(item) {
            result.push(item.clone());
        }
    }
    result
}

pub fn normalize_strings(strings: &[String]) -> Vec<String> {
    strings.iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn clean_numeric_data(numbers: &[f64]) -> Vec<f64> {
    numbers.iter()
        .filter(|&&n| n.is_finite() && !n.is_nan())
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let data = vec![1, 2, 2, 3, 4, 4, 5];
        let cleaned = remove_duplicates(&data);
        assert_eq!(cleaned, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_normalize_strings() {
        let strings = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "  TEST  ".to_string(),
        ];
        let normalized = normalize_strings(&strings);
        assert_eq!(normalized, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_clean_numeric_data() {
        let numbers = vec![1.0, f64::NAN, 2.0, f64::INFINITY, 3.0];
        let cleaned = clean_numeric_data(&numbers);
        assert_eq!(cleaned, vec![1.0, 2.0, 3.0]);
    }
}