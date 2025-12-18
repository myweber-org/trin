
use std::collections::HashSet;

pub struct DataCleaner {
    data: Vec<String>,
}

impl DataCleaner {
    pub fn new(data: Vec<String>) -> Self {
        DataCleaner { data }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
        self
    }

    pub fn normalize(&mut self) -> &mut Self {
        for item in &mut self.data {
            *item = item.trim().to_lowercase();
        }
        self
    }

    pub fn filter_empty(&mut self) -> &mut Self {
        self.data.retain(|item| !item.is_empty());
        self
    }

    pub fn get_data(&self) -> &Vec<String> {
        &self.data
    }

    pub fn process(&mut self) -> &Vec<String> {
        self.deduplicate()
            .normalize()
            .filter_empty()
            .get_data()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaning() {
        let raw_data = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "Banana".to_string(),
            "".to_string(),
            "banana".to_string(),
            "  CHERRY  ".to_string(),
        ];

        let mut cleaner = DataCleaner::new(raw_data);
        let result = cleaner.process();

        assert_eq!(result.len(), 3);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
        assert!(result.contains(&"cherry".to_string()));
    }
}