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

    pub fn clean(&self, data: Vec<String>) -> Vec<String> {
        let mut processed_data = data;

        if self.normalize_case {
            processed_data = processed_data
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect();
        }

        if self.remove_duplicates {
            let unique_set: HashSet<String> = processed_data.into_iter().collect();
            processed_data = unique_set.into_iter().collect();
        }

        processed_data.sort();
        processed_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_with_duplicates() {
        let cleaner = DataCleaner::new(true, false);
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        let result = cleaner.clean(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_cleaner_with_case_normalization() {
        let cleaner = DataCleaner::new(false, true);
        let input = vec!["Apple".to_string(), "BANANA".to_string(), "cherry".to_string()];
        let result = cleaner.clean(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }
}