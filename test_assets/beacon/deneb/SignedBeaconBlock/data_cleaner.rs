use std::collections::HashSet;

pub fn clean_data<T: Eq + std::hash::Hash + Clone>(
    data: Vec<T>,
    invalid_set: &HashSet<T>,
) -> Vec<T> {
    data.into_iter()
        .filter(|item| !invalid_set.contains(item))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let data = vec![1, 2, 3, 4, 5];
        let invalid: HashSet<i32> = [2, 4].iter().cloned().collect();
        let cleaned = clean_data(data, &invalid);
        assert_eq!(cleaned, vec![1, 3, 5]);
    }
}