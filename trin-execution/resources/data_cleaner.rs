
use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub normalize_whitespace: bool,
    pub trim_strings: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_duplicates: true,
            normalize_whitespace: true,
            trim_strings: true,
        }
    }

    pub fn clean_dataset(&self, data: Vec<String>) -> Vec<String> {
        let mut processed: Vec<String> = data
            .into_iter()
            .map(|item| {
                let mut result = item;
                
                if self.trim_strings {
                    result = result.trim().to_string();
                }
                
                if self.normalize_whitespace {
                    result = result.split_whitespace().collect::<Vec<&str>>().join(" ");
                }
                
                result
            })
            .collect();

        if self.remove_duplicates {
            let unique_set: HashSet<String> = processed.drain(..).collect();
            processed = unique_set.into_iter().collect();
        }

        processed.sort();
        processed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_duplicates() {
        let cleaner = DataCleaner::new();
        let data = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 3);
        assert_eq!(cleaned, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_cleaner_normalizes_whitespace() {
        let cleaner = DataCleaner::new();
        let data = vec!["  hello   world  ".to_string()];
        
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned[0], "hello world");
    }
}