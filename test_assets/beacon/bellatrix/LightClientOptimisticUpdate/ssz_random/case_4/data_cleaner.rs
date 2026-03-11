
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
}use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<&str>) -> Vec<String> {
        data.iter()
            .filter(|&&item| self.deduplicate(item))
            .map(|&item| self.normalize_text(item))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["Apple", "apple", "APPLE", "Banana", "banana"];
        let cleaned = cleaner.clean_dataset(data);
        
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }
}