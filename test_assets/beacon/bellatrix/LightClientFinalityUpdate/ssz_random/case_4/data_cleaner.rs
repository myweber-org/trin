use std::collections::HashSet;

pub struct DataCleaner {
    data: Vec<Vec<Option<String>>>,
}

impl DataCleaner {
    pub fn new(data: Vec<Vec<Option<String>>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self) {
        self.data.retain(|row| {
            row.iter().all(|cell| cell.is_some())
        });
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| {
            let key: Vec<&str> = row.iter()
                .map(|cell| cell.as_deref().unwrap_or(""))
                .collect();
            seen.insert(key)
        });
    }

    pub fn get_data(&self) -> &Vec<Vec<Option<String>>> {
        &self.data
    }

    pub fn clean(&mut self) {
        self.remove_null_rows();
        self.deduplicate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_nulls() {
        let data = vec![
            vec![Some("A".to_string()), Some("B".to_string())],
            vec![Some("C".to_string()), None],
            vec![Some("E".to_string()), Some("F".to_string())],
        ];
        
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_rows();
        
        assert_eq!(cleaner.get_data().len(), 2);
    }

    #[test]
    fn test_cleaner_deduplicates() {
        let data = vec![
            vec![Some("X".to_string()), Some("Y".to_string())],
            vec![Some("X".to_string()), Some("Y".to_string())],
            vec![Some("Z".to_string()), Some("W".to_string())],
        ];
        
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        
        assert_eq!(cleaner.get_data().len(), 2);
    }
}