use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    
    result
}

pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn clean_numeric_data(numbers: Vec<f64>) -> Vec<f64> {
    numbers
        .into_iter()
        .filter(|&n| n.is_finite())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 3, 3];
        let result = deduplicate(input);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_clean_numeric_data() {
        let input = vec![1.0, f64::NAN, 2.0, f64::INFINITY];
        let result = clean_numeric_data(input);
        assert_eq!(result, vec![1.0, 2.0]);
    }
}