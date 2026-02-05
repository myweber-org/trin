
use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<Vec<T>>,
}

impl<T> DataCleaner<T>
where
    T: Clone + PartialEq + Eq + std::hash::Hash,
{
    pub fn new(data: Vec<Vec<T>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self, null_value: &T) {
        self.data.retain(|row| !row.contains(null_value));
    }

    pub fn deduplicate_rows(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| seen.insert(row.clone()));
    }

    pub fn get_data(&self) -> &Vec<Vec<T>> {
        &self.data
    }

    pub fn into_data(self) -> Vec<Vec<T>> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_null_rows() {
        let data = vec![
            vec!["A", "B", "C"],
            vec!["X", "NULL", "Z"],
            vec!["D", "E", "F"],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_rows(&"NULL");
        
        assert_eq!(cleaner.get_data().len(), 2);
        assert_eq!(cleaner.get_data()[0], vec!["A", "B", "C"]);
        assert_eq!(cleaner.get_data()[1], vec!["D", "E", "F"]);
    }

    #[test]
    fn test_deduplicate_rows() {
        let data = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![1, 2, 3],
            vec![7, 8, 9],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate_rows();
        
        assert_eq!(cleaner.get_data().len(), 3);
    }
}use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn filter_empty_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 4, 4, 5];
        let result = deduplicate(input);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_filter_empty_strings() {
        let input = vec!["hello".to_string(), "".to_string(), "world".to_string()];
        let result = filter_empty_strings(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }
}