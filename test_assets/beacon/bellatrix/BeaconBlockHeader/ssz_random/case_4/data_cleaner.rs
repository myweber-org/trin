use std::collections::HashSet;

pub struct DataCleaner {
    data: Vec<Vec<String>>,
}

impl DataCleaner {
    pub fn new(data: Vec<Vec<String>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self) {
        self.data.retain(|row| !row.iter().any(|cell| cell.trim().is_empty()));
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| {
            let key: String = row.iter().map(|s| s.trim()).collect();
            seen.insert(key)
        });
    }

    pub fn get_cleaned_data(&self) -> &Vec<Vec<String>> {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_empty_rows() {
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["".to_string(), "c".to_string()],
            vec!["d".to_string(), "".to_string()],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_rows();
        assert_eq!(cleaner.get_cleaned_data().len(), 1);
    }

    #[test]
    fn test_cleaner_deduplicates() {
        let data = vec![
            vec!["x".to_string(), "y".to_string()],
            vec!["x".to_string(), "y".to_string()],
            vec!["a".to_string(), "b".to_string()],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        assert_eq!(cleaner.get_cleaned_data().len(), 2);
    }
}