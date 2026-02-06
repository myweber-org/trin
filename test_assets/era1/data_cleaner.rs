
use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T: Clone + Eq + std::hash::Hash> DataCleaner<T> {
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_values(&mut self) -> &mut Self {
        self.data.retain(|item| !Self::is_null(item));
        self
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
        self
    }

    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }

    fn is_null(item: &T) -> bool {
        // For demonstration, treat empty strings as null
        if let Some(s) = (item as &dyn std::any::Any).downcast_ref::<String>() {
            s.is_empty()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_null_values() {
        let data = vec![
            "valid".to_string(),
            "".to_string(),
            "another".to_string(),
            "".to_string(),
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_values();
        assert_eq!(cleaner.get_data().len(), 2);
    }

    #[test]
    fn test_deduplicate() {
        let data = vec![1, 2, 2, 3, 1, 4];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        assert_eq!(cleaner.get_data().len(), 4);
    }

    #[test]
    fn test_chain_operations() {
        let data = vec![
            "apple".to_string(),
            "".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "".to_string(),
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_values().deduplicate();
        assert_eq!(cleaner.get_data().len(), 2);
    }
}