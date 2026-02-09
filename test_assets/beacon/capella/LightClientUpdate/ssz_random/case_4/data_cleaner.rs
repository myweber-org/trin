use std::collections::HashSet;

pub fn clean_and_sort_data(input: Vec<String>) -> Vec<String> {
    let mut unique_items: HashSet<String> = input.into_iter().collect();
    let mut sorted_items: Vec<String> = unique_items.into_iter().collect();
    sorted_items.sort();
    sorted_items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_sort_data() {
        let input = vec![
            "zebra".to_string(),
            "apple".to_string(),
            "zebra".to_string(),
            "banana".to_string(),
        ];
        let result = clean_and_sort_data(input);
        assert_eq!(result, vec!["apple", "banana", "zebra"]);
    }
}