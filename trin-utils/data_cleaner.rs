
use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Clone + Eq + Hash,
{
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
        self
    }

    pub fn normalize(&mut self) -> &mut Self
    where
        T: ToString,
    {
        self.data.sort();
        self
    }

    pub fn filter<F>(&mut self, predicate: F) -> &mut Self
    where
        F: Fn(&T) -> bool,
    {
        self.data.retain(predicate);
        self
    }

    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }
}

pub fn clean_strings(strings: Vec<String>) -> Vec<String> {
    let mut cleaner = DataCleaner::new(strings);
    cleaner
        .deduplicate()
        .normalize()
        .filter(|s| !s.trim().is_empty())
        .into_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let data = vec![1, 2, 2, 3, 3, 3];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        assert_eq!(cleaner.get_data(), &vec![1, 2, 3]);
    }

    #[test]
    fn test_string_cleaning() {
        let strings = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "".to_string(),
            "  ".to_string(),
            "banana".to_string(),
        ];
        let cleaned = clean_strings(strings);
        assert_eq!(cleaned, vec!["apple".to_string(), "banana".to_string()]);
    }
}