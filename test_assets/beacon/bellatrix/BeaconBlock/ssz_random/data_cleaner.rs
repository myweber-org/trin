use std::collections::HashSet;

pub fn clean_and_sort_data(data: Vec<String>) -> Vec<String> {
    let unique_data: HashSet<String> = data.into_iter().collect();
    let mut sorted_data: Vec<String> = unique_data.into_iter().collect();
    sorted_data.sort();
    sorted_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_sort_data() {
        let input = vec![
            "banana".to_string(),
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
            "apple".to_string(),
        ];
        let result = clean_and_sort_data(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }
}