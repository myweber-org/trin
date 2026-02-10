use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataCleaner {
    pub delimiter: char,
    pub skip_header: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            delimiter: ',',
            skip_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn clean_csv(&self, file_path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut cleaned_data = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if self.skip_header && index == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                cleaned_data.push(fields);
            }
        }

        Ok(cleaned_data)
    }

    pub fn convert_column_to_numeric(&self, data: &[Vec<String>], column_index: usize) -> Vec<Option<f64>> {
        data.iter()
            .map(|row| {
                if column_index < row.len() {
                    row[column_index].parse::<f64>().ok()
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn filter_valid_rows(&self, data: Vec<Vec<String>>) -> Vec<Vec<String>> {
        data.into_iter()
            .filter(|row| !row.is_empty() && row.iter().any(|field| !field.is_empty()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();

        let cleaner = DataCleaner::new();
        let result = cleaner.clean_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "25", "New York"]);
    }

    #[test]
    fn test_convert_column_to_numeric() {
        let data = vec![
            vec!["10".to_string(), "20".to_string()],
            vec!["invalid".to_string(), "30".to_string()],
            vec!["40".to_string()],
        ];
        
        let cleaner = DataCleaner::new();
        let numeric = cleaner.convert_column_to_numeric(&data, 0);
        
        assert_eq!(numeric, vec![Some(10.0), None, Some(40.0)]);
    }
}use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn merge_and_clean<T: Eq + Hash + Clone>(list1: Vec<T>, list2: Vec<T>) -> Vec<T> {
    let mut combined = list1;
    combined.extend(list2);
    deduplicate(combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 4, 4, 5];
        let result = deduplicate(input);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec![
            "  Hello  ".to_string(),
            "WORLD".to_string(),
            "".to_string(),
            "  test  ".to_string(),
        ];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_merge_and_clean() {
        let list1 = vec!["a", "b", "c"];
        let list2 = vec!["b", "c", "d"];
        let result = merge_and_clean(
            list1.into_iter().map(String::from).collect(),
            list2.into_iter().map(String::from).collect(),
        );
        assert_eq!(result, vec!["a", "b", "c", "d"]);
    }
}
use std::collections::HashMap;

pub struct DataCleaner {
    pub remove_nulls: bool,
    pub trim_whitespace: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_nulls: true,
            trim_whitespace: true,
        }
    }

    pub fn clean_string(&self, input: Option<String>) -> Option<String> {
        match input {
            Some(mut s) => {
                if self.trim_whitespace {
                    s = s.trim().to_string();
                }
                if s.is_empty() && self.remove_nulls {
                    None
                } else {
                    Some(s)
                }
            }
            None => None,
        }
    }

    pub fn clean_hashmap(&self, data: HashMap<String, Option<String>>) -> HashMap<String, String> {
        data.into_iter()
            .filter_map(|(key, value)| {
                self.clean_string(value).map(|clean_value| (key, clean_value))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_clean_string() {
        let cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.clean_string(Some("  hello  ".to_string())), Some("hello".to_string()));
        assert_eq!(cleaner.clean_string(Some("".to_string())), None);
        assert_eq!(cleaner.clean_string(Some("   ".to_string())), None);
        assert_eq!(cleaner.clean_string(None), None);
    }

    #[test]
    fn test_clean_hashmap() {
        let cleaner = DataCleaner::new();
        let mut input = HashMap::new();
        input.insert("name".to_string(), Some("  john  ".to_string()));
        input.insert("email".to_string(), Some("".to_string()));
        input.insert("age".to_string(), None);
        input.insert("city".to_string(), Some("new york".to_string()));

        let result = cleaner.clean_hashmap(input);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("name"), Some(&"john".to_string()));
        assert_eq!(result.get("city"), Some(&"new york".to_string()));
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in rdr.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping invalid record: {}", e);
                continue;
            }
        };

        if record.age > 120 {
            eprintln!("Invalid age {} for record {}, skipping", record.age, record.id);
            continue;
        }

        wtr.serialize(&record)?;
    }

    wtr.flush()?;
    println!("Cleaned data written to {}", output_path);
    Ok(())
}

fn main() {
    if let Err(e) = clean_csv("input.csv", "output.csv") {
        eprintln!("Error cleaning CSV: {}", e);
        std::process::exit(1);
    }
}