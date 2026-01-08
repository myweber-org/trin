
use std::collections::HashSet;

pub fn clean_dataset<T: Eq + std::hash::Hash + Clone>(
    data: &[T],
    invalid_entries: &HashSet<T>,
) -> Vec<T> {
    data.iter()
        .filter(|entry| !invalid_entries.contains(entry))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_dataset() {
        let data = vec![1, 2, 3, 4, 5];
        let invalid = HashSet::from([2, 4]);
        let cleaned = clean_dataset(&data, &invalid);
        assert_eq!(cleaned, vec![1, 3, 5]);
    }

    #[test]
    fn test_empty_invalid_set() {
        let data = vec!["apple", "banana", "cherry"];
        let invalid = HashSet::new();
        let cleaned = clean_dataset(&data, &invalid);
        assert_eq!(cleaned, data);
    }

    #[test]
    fn test_all_invalid() {
        let data = vec![10, 20, 30];
        let invalid = HashSet::from([10, 20, 30]);
        let cleaned = clean_dataset(&data, &invalid);
        assert!(cleaned.is_empty());
    }
}