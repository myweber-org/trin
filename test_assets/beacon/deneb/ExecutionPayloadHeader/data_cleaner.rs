
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
                
                for value in column_data.iter_mut() {
                    match value {
                        Some(text) => {
                            let trimmed = text.trim().to_string();
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
            
            for (column_name, column_values) in &self.data {
                if row_index >= column_values.len() || column_values[row_index].is_none() {
                    row_has_null = true;
                    break;
                }
            }
            
            if !row_has_null {
                for (column_name, column_values) in &self.data {
                    if let Some(value) = &column_values[row_index] {
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

pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_string() {
        let input = "Hello, World! 123";
        let expected = "Hello World 123";
        assert_eq!(sanitize_string(input), expected);
    }

    #[test]
    fn test_clean_column() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_column(
            "names",
            vec![
                Some("  Alice  ".to_string()),
                None,
                Some("Bob".to_string()),
                Some("  ".to_string()),
            ],
        );
        
        let cleaned = cleaner.clean_column("names").unwrap();
        assert_eq!(cleaned, vec!["Alice", "Bob"]);
    }
}