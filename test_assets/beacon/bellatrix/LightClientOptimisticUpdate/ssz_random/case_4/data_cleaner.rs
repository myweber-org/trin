
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

    pub fn normalize<F>(&mut self, normalizer: F) -> &mut Self
    where
        F: Fn(&T) -> T,
    {
        self.data = self.data.iter().map(|item| normalizer(item)).collect();
        self
    }

    pub fn filter<F>(&mut self, predicate: F) -> &mut Self
    where
        F: Fn(&T) -> bool,
    {
        self.data.retain(|item| predicate(item));
        self
    }

    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }
}

pub fn clean_string_data(strings: Vec<String>) -> Vec<String> {
    let mut cleaner = DataCleaner::new(strings);
    cleaner
        .normalize(|s| s.trim().to_lowercase())
        .deduplicate()
        .filter(|s| !s.is_empty())
        .into_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let data = vec![1, 2, 2, 3, 3, 3];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        assert_eq!(cleaner.get_data(), &vec![1, 2, 3]);
    }

    #[test]
    fn test_normalize() {
        let data = vec!["HELLO", "World", "rust"];
        let mut cleaner = DataCleaner::new(data.into_iter().map(String::from).collect());
        cleaner.normalize(|s| s.to_lowercase());
        assert_eq!(
            cleaner.get_data(),
            &vec!["hello".to_string(), "world".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn test_clean_string_data() {
        let input = vec![
            "  HELLO  ".to_string(),
            "hello".to_string(),
            "".to_string(),
            "WORLD".to_string(),
            "world".to_string(),
        ];
        let result = clean_string_data(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }
}