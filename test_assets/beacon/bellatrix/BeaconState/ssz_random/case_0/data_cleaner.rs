
use std::collections::HashSet;

pub fn filter_valid_data<T: Eq + std::hash::Hash + Clone>(
    data: &[T],
    valid_set: &HashSet<T>,
) -> Vec<T> {
    data.iter()
        .filter(|item| valid_set.contains(item))
        .cloned()
        .collect()
}

pub fn remove_duplicates<T: Eq + std::hash::Hash + Clone>(data: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for item in data {
        if seen.insert(item.clone()) {
            result.push(item.clone());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_valid_data() {
        let data = vec![1, 2, 3, 4, 5];
        let valid_set: HashSet<i32> = [2, 4, 6].iter().cloned().collect();

        let filtered = filter_valid_data(&data, &valid_set);
        assert_eq!(filtered, vec![2, 4]);
    }

    #[test]
    fn test_remove_duplicates() {
        let data = vec![1, 2, 2, 3, 3, 3, 4];
        let unique = remove_duplicates(&data);
        assert_eq!(unique, vec![1, 2, 3, 4]);
    }
}