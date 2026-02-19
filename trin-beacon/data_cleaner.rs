
use std::collections::HashMap;

pub struct DataCleaner {
    data: HashMap<String, Vec<Option<String>>>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            data: HashMap::new(),
        }
    }

    pub fn add_column(&mut self, column_name: &str, values: Vec<Option<String>>) {
        self.data.insert(column_name.to_string(), values);
    }

    pub fn clean_column(&mut self, column_name: &str) -> Result<Vec<String>, String> {
        match self.data.get_mut(column_name) {
            Some(column_data) => {
                let mut cleaned = Vec::new();
                
                for value in column_data.iter() {
                    match value {
                        Some(v) => {
                            let trimmed = v.trim().to_string();
                            if !trimmed.is_empty() {
                                cleaned.push(trimmed);
                            }
                        }
                        None => continue,
                    }
                }
                
                Ok(cleaned)
            }
            None => Err(format!("Column '{}' not found", column_name)),
        }
    }

    pub fn remove_null_rows(&mut self) -> HashMap<String, Vec<String>> {
        let mut cleaned_data = HashMap::new();
        let mut row_count = 0;
        
        if let Some(first_column) = self.data.keys().next() {
            if let Some(first_values) = self.data.get(first_column) {
                row_count = first_values.len();
            }
        }

        for row_index in 0..row_count {
            let mut row_has_null = false;
            
            for (column_name, column_data) in &self.data {
                if row_index >= column_data.len() || column_data[row_index].is_none() {
                    row_has_null = true;
                    break;
                }
            }

            if !row_has_null {
                for (column_name, column_data) in &self.data {
                    if let Some(value) = &column_data[row_index] {
                        cleaned_data
                            .entry(column_name.clone())
                            .or_insert_with(Vec::new)
                            .push(value.trim().to_string());
                    }
                }
            }
        }

        cleaned_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_column() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_column(
            "names",
            vec![
                Some("  John  ".to_string()),
                Some("".to_string()),
                None,
                Some("Jane".to_string()),
            ],
        );

        let cleaned = cleaner.clean_column("names").unwrap();
        assert_eq!(cleaned, vec!["John", "Jane"]);
    }

    #[test]
    fn test_remove_null_rows() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_column(
            "id",
            vec![
                Some("1".to_string()),
                Some("2".to_string()),
                None,
                Some("4".to_string()),
            ],
        );
        cleaner.add_column(
            "value",
            vec![
                Some("A".to_string()),
                Some("B".to_string()),
                Some("C".to_string()),
                Some("D".to_string()),
            ],
        );

        let cleaned = cleaner.remove_null_rows();
        assert_eq!(cleaned.get("id").unwrap(), &vec!["1", "2", "4"]);
        assert_eq!(cleaned.get("value").unwrap(), &vec!["A", "B", "D"]);
    }
}