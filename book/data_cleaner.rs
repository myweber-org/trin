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
        let filtered_data: Vec<T> = self
            .data
            .into_iter()
            .filter(|item| *item != None)
            .collect();
        DataCleaner {
            data: filtered_data,
        }
    }

    pub fn deduplicate(self) -> Self
    where
        T: Eq + std::hash::Hash + Clone,
    {
        let mut seen = HashSet::new();
        let unique_data: Vec<T> = self
            .data
            .into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect();
        DataCleaner {
            data: unique_data,
        }
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
    cleaner
        .remove_nulls()
        .deduplicate()
        .get_data()
}use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn deduplicate(&mut self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut unique_records = Vec::new();

        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                unique_records.push(record);
            }
        }

        self.records = unique_records.clone();
        unique_records
    }

    pub fn normalize_whitespace(&mut self) {
        for record in self.records.iter_mut() {
            let normalized = record
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ");
            *record = normalized;
        }
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("another".to_string());

        let deduped = cleaner.deduplicate();
        assert_eq!(deduped.len(), 2);
        assert_eq!(cleaner.get_records().len(), 2);
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  multiple   spaces   ".to_string());
        cleaner.normalize_whitespace();

        assert_eq!(cleaner.get_records()[0], "multiple spaces");
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

    pub fn normalize_text(&self, text: &str) -> String {
        text.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
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

    pub fn process_batch(&mut self, items: Vec<&str>) -> Vec<String> {
        items
            .into_iter()
            .filter(|item| self.deduplicate(item))
            .map(|item| self.normalize_text(item))
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
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World!  "), "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("hello"));
        assert!(!cleaner.deduplicate("HELLO"));
        assert!(cleaner.deduplicate("world"));
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let items = vec!["Apple", "apple", "Banana", "  banana  "];
        let result = cleaner.process_batch(items);
        assert_eq!(result, vec!["apple", "banana"]);
    }
}