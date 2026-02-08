
use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_values(&mut self, null_value: T) -> &mut Self {
        self.data.retain(|item| *item != null_value);
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
}

pub fn clean_dataset<T>(data: Vec<T>, null_value: T) -> Vec<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    let mut cleaner = DataCleaner::new(data);
    cleaner
        .remove_null_values(null_value)
        .deduplicate()
        .into_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_dataset() {
        let data = vec![1, 2, 2, 3, 0, 4, 0, 5];
        let cleaned = clean_dataset(data, 0);
        assert_eq!(cleaned, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_remove_null_values() {
        let mut cleaner = DataCleaner::new(vec!["a", "", "b", "", "c"]);
        cleaner.remove_null_values("");
        assert_eq!(cleaner.get_data(), &vec!["a", "b", "c"]);
    }

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new(vec![1.5, 2.0, 1.5, 3.0, 2.0]);
        cleaner.deduplicate();
        let mut result = cleaner.into_data();
        result.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(result, vec![1.5, 2.0, 3.0]);
    }
}