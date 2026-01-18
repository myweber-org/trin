use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub normalize_case: bool,
}

impl DataCleaner {
    pub fn new(remove_duplicates: bool, normalize_case: bool) -> Self {
        DataCleaner {
            remove_duplicates,
            normalize_case,
        }
    }

    pub fn clean(&self, data: Vec<String>) -> Vec<String> {
        let mut processed_data = data;

        if self.normalize_case {
            processed_data = processed_data
                .iter()
                .map(|s| s.to_lowercase())
                .collect();
        }

        if self.remove_duplicates {
            let unique_set: HashSet<String> = processed_data.into_iter().collect();
            processed_data = unique_set.into_iter().collect();
        }

        processed_data.sort();
        processed_data
    }

    pub fn validate_email(&self, email: &str) -> bool {
        let email = email.trim();
        let parts: Vec<&str> = email.split('@').collect();
        
        if parts.len() != 2 {
            return false;
        }

        let local_part = parts[0];
        let domain_part = parts[1];

        !local_part.is_empty() && 
        !domain_part.is_empty() && 
        domain_part.contains('.') &&
        !email.contains(' ')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_with_duplicates() {
        let cleaner = DataCleaner::new(true, false);
        let data = vec![
            "Apple".to_string(),
            "banana".to_string(),
            "Apple".to_string(),
            "Cherry".to_string(),
        ];
        
        let result = cleaner.clean(data);
        assert_eq!(result, vec!["Apple", "Cherry", "banana"]);
    }

    #[test]
    fn test_clean_with_normalization() {
        let cleaner = DataCleaner::new(false, true);
        let data = vec![
            "APPLE".to_string(),
            "Banana".to_string(),
            "cherry".to_string(),
        ];
        
        let result = cleaner.clean(data);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_email_validation() {
        let cleaner = DataCleaner::new(false, false);
        
        assert!(cleaner.validate_email("test@example.com"));
        assert!(!cleaner.validate_email("invalid-email"));
        assert!(!cleaner.validate_email("test@com"));
        assert!(!cleaner.validate_email("test @example.com"));
    }
}use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);
    
    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;
    
    for result in rdr.records() {
        let record = result?;
        let filtered_record: Vec<&str> = record
            .iter()
            .filter(|field| !field.trim().is_empty())
            .collect();
        
        if filtered_record.len() == headers.len() {
            wtr.write_record(&filtered_record)?;
        }
    }
    
    wtr.flush()?;
    Ok(())
}