
use std::collections::HashMap;

pub struct DataCleaner {
    pub null_values: Vec<String>,
    pub string_normalizations: HashMap<String, String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            null_values: vec![
                "null".to_string(),
                "NULL".to_string(),
                "".to_string(),
                "N/A".to_string(),
                "n/a".to_string(),
            ],
            string_normalizations: HashMap::from([
                ("  ".to_string(), " ".to_string()),
                ("\t".to_string(), " ".to_string()),
                ("\n".to_string(), " ".to_string()),
            ]),
        }
    }

    pub fn clean_string(&self, input: &str) -> Option<String> {
        if self.null_values.contains(&input.to_string()) {
            return None;
        }

        let mut result = input.to_string();
        for (pattern, replacement) in &self.string_normalizations {
            result = result.replace(pattern, replacement);
        }

        result = result.trim().to_string();
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn clean_vector(&self, data: Vec<&str>) -> Vec<String> {
        data.iter()
            .filter_map(|&item| self.clean_string(item))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.clean_string("hello"), Some("hello".to_string()));
        assert_eq!(cleaner.clean_string(""), None);
        assert_eq!(cleaner.clean_string("null"), None);
        assert_eq!(cleaner.clean_string("  hello  "), Some("hello".to_string()));
        assert_eq!(cleaner.clean_string("hello\tworld"), Some("hello world".to_string()));
    }

    #[test]
    fn test_clean_vector() {
        let cleaner = DataCleaner::new();
        let data = vec!["hello", "", "null", "  test  "];
        let cleaned = cleaner.clean_vector(data);
        assert_eq!(cleaned, vec!["hello".to_string(), "test".to_string()]);
    }
}use std::collections::HashSet;

pub fn clean_and_sort_data<T: Ord + Clone>(data: &[T]) -> Vec<T> {
    let unique_items: HashSet<_> = data.iter().cloned().collect();
    let mut result: Vec<T> = unique_items.into_iter().collect();
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_sort_numbers() {
        let input = vec![5, 2, 8, 2, 5, 1, 9];
        let expected = vec![1, 2, 5, 8, 9];
        assert_eq!(clean_and_sort_data(&input), expected);
    }

    #[test]
    fn test_clean_and_sort_strings() {
        let input = vec!["banana", "apple", "cherry", "apple", "banana"];
        let expected = vec!["apple", "banana", "cherry"];
        assert_eq!(clean_and_sort_data(&input), expected);
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<i32> = vec![];
        let expected: Vec<i32> = vec![];
        assert_eq!(clean_and_sort_data(&input), expected);
    }
}