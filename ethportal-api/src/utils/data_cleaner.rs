
use std::collections::HashSet;

pub fn clean_string_data(input: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for item in input {
        let normalized = item.trim().to_lowercase();
        if !normalized.is_empty() && seen.insert(normalized.clone()) {
            result.push(normalized);
        }
    }

    result.sort();
    result
}

pub fn remove_numeric_duplicates(input: Vec<f64>) -> Vec<f64> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for &num in &input {
        if seen.insert((num * 1000.0).round() as i64) {
            result.push(num);
        }
    }

    result.sort_by(|a, b| a.partial_cmp(b).unwrap());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string_data() {
        let input = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "BANANA".to_string(),
            "banana ".to_string(),
            "".to_string(),
            "  Cherry  ".to_string(),
        ];
        
        let result = clean_string_data(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_remove_numeric_duplicates() {
        let input = vec![3.141, 3.1415, 2.718, 3.141, 2.718];
        let result = remove_numeric_duplicates(input);
        assert_eq!(result, vec![2.718, 3.141, 3.1415]);
    }
}