
use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub normalize_case: bool,
}

impl DataCleaner {
    pub fn new(remove_duplicates: bool, normalize_case: bool) -> Self {
        DataCleaner {
            remove_duplicates,
            normalize_case,
        }
    }

    pub fn clean_strings(&self, input: Vec<String>) -> Vec<String> {
        let mut processed: Vec<String> = input
            .into_iter()
            .map(|s| {
                if self.normalize_case {
                    s.to_lowercase()
                } else {
                    s
                }
            })
            .collect();

        if self.remove_duplicates {
            let mut seen = HashSet::new();
            processed.retain(|s| seen.insert(s.clone()));
        }

        processed
    }

    pub fn deduplicate_numbers(numbers: &[i32]) -> Vec<i32> {
        let mut unique = HashSet::new();
        numbers
            .iter()
            .filter(|&&n| unique.insert(n))
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_strings_with_duplicates() {
        let cleaner = DataCleaner::new(true, false);
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        let result = cleaner.clean_strings(input);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_clean_strings_with_case_normalization() {
        let cleaner = DataCleaner::new(false, true);
        let input = vec!["Apple".to_string(), "BANANA".to_string()];
        let result = cleaner.clean_strings(input);
        assert_eq!(result[0], "apple");
        assert_eq!(result[1], "banana");
    }

    #[test]
    fn test_deduplicate_numbers() {
        let numbers = vec![1, 2, 3, 2, 1, 4];
        let result = DataCleaner::deduplicate_numbers(&numbers);
        assert_eq!(result.len(), 4);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
        assert!(result.contains(&4));
    }
}