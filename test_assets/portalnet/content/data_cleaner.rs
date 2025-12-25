use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item) {
            result.push(item.clone());
        }
    }
    result
}

pub fn normalize_strings(strings: &[String]) -> Vec<String> {
    strings
        .iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn clean_data(strings: &[String]) -> Vec<String> {
    let normalized = normalize_strings(strings);
    deduplicate(&normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 4, 4, 4, 5];
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(deduplicate(&input), expected);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec![
            "  Hello  ".to_string(),
            "WORLD".to_string(),
            "  TeSt  ".to_string(),
        ];
        let expected = vec!["hello".to_string(), "world".to_string(), "test".to_string()];
        assert_eq!(normalize_strings(&input), expected);
    }

    #[test]
    fn test_clean_data() {
        let input = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "  BANANA  ".to_string(),
            "Banana".to_string(),
            "  APPLE  ".to_string(),
        ];
        let expected = vec!["apple".to_string(), "banana".to_string()];
        assert_eq!(clean_data(&input), expected);
    }
}