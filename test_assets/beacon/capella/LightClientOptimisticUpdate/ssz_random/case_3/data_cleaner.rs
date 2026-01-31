use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T> {
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_nulls(self) -> Self
    where
        T: PartialEq,
    {
        let filtered_data: Vec<T> = self.data.into_iter().filter(|item| item != &None).collect();
        DataCleaner { data: filtered_data }
    }

    pub fn remove_duplicates(self) -> Self
    where
        T: Eq + std::hash::Hash + Clone,
    {
        let unique_set: HashSet<T> = self.data.into_iter().collect();
        let unique_data: Vec<T> = unique_set.into_iter().collect();
        DataCleaner { data: unique_data }
    }

    pub fn get_data(self) -> Vec<T> {
        self.data
    }
}

pub fn clean_dataset<T>(data: Vec<T>) -> Vec<T>
where
    T: Eq + std::hash::Hash + Clone + PartialEq,
{
    let cleaner = DataCleaner::new(data);
    cleaner.remove_nulls().remove_duplicates().get_data()
}
use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    unique_ids: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_ids: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, id: &str) -> bool {
        self.unique_ids.insert(id.to_string())
    }

    pub fn validate_email(email: &str) -> Result<(), Box<dyn Error>> {
        if email.contains('@') && email.contains('.') {
            Ok(())
        } else {
            Err("Invalid email format".into())
        }
    }

    pub fn normalize_string(input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn remove_duplicates<T: Eq + std::hash::Hash + Clone>(items: Vec<T>) -> Vec<T> {
        let mut seen = HashSet::new();
        items
            .into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("user123"));
        assert!(!cleaner.deduplicate("user123"));
        assert!(cleaner.deduplicate("user456"));
    }

    #[test]
    fn test_validate_email() {
        assert!(DataCleaner::validate_email("test@example.com").is_ok());
        assert!(DataCleaner::validate_email("invalid").is_err());
    }

    #[test]
    fn test_normalize_string() {
        assert_eq!(DataCleaner::normalize_string("  TEST  "), "test");
        assert_eq!(DataCleaner::normalize_string("MixedCase"), "mixedcase");
    }

    #[test]
    fn test_remove_duplicates() {
        let items = vec![1, 2, 2, 3, 1, 4];
        let unique = DataCleaner::remove_duplicates(items);
        assert_eq!(unique, vec![1, 2, 3, 4]);
    }
}