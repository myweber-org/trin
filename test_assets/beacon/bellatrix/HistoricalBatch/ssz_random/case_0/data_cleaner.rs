
use std::collections::HashSet;

pub struct DataCleaner;

impl DataCleaner {
    pub fn clean_strings(strings: Vec<String>) -> Vec<String> {
        let mut unique_strings: HashSet<String> = strings.into_iter().collect();
        let mut sorted_strings: Vec<String> = unique_strings.into_iter().collect();
        sorted_strings.sort();
        sorted_strings
    }
    
    pub fn clean_integers(numbers: Vec<i32>) -> Vec<i32> {
        let mut unique_numbers: HashSet<i32> = numbers.into_iter().collect();
        let mut sorted_numbers: Vec<i32> = unique_numbers.into_iter().collect();
        sorted_numbers.sort();
        sorted_numbers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_strings() {
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
            "banana".to_string(),
        ];
        
        let result = DataCleaner::clean_strings(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_clean_integers() {
        let input = vec![5, 2, 8, 2, 5, 1, 9];
        let result = DataCleaner::clean_integers(input);
        assert_eq!(result, vec![1, 2, 5, 8, 9]);
    }
}