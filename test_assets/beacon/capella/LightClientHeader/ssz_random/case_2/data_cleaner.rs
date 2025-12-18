
use std::collections::HashMap;

pub struct DataCleaner {
    pub column_defaults: HashMap<String, String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            column_defaults: HashMap::new(),
        }
    }

    pub fn set_default(&mut self, column: &str, default_value: &str) {
        self.column_defaults.insert(column.to_string(), default_value.to_string());
    }

    pub fn clean_row(&self, row: &mut HashMap<String, String>) {
        for (column, default_value) in &self.column_defaults {
            if !row.contains_key(column) || row.get(column).unwrap().trim().is_empty() {
                row.insert(column.clone(), default_value.clone());
            }
        }

        for (_, value) in row.iter_mut() {
            *value = value.trim().to_lowercase();
        }
    }

    pub fn clean_dataset(&self, dataset: &mut Vec<HashMap<String, String>>) {
        for row in dataset {
            self.clean_row(row);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let mut cleaner = DataCleaner::new();
        cleaner.set_default("name", "unknown");
        cleaner.set_default("age", "0");

        let mut dataset = vec![
            HashMap::from([
                ("name".to_string(), "  JOHN  ".to_string()),
                ("age".to_string(), "".to_string()),
            ]),
            HashMap::from([
                ("name".to_string(), "".to_string()),
                ("age".to_string(), "25".to_string()),
            ]),
        ];

        cleaner.clean_dataset(&mut dataset);

        assert_eq!(dataset[0]["name"], "john");
        assert_eq!(dataset[0]["age"], "0");
        assert_eq!(dataset[1]["name"], "unknown");
        assert_eq!(dataset[1]["age"], "25");
    }
}