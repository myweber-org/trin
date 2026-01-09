
use std::collections::HashSet;
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
        .collect()
}

pub fn filter_by_length(strings: Vec<String>, min_len: usize) -> Vec<String> {
    strings
        .into_iter()
        .filter(|s| s.len() >= min_len)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 4, 4, 4, 5];
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(deduplicate(input), expected);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_filter_by_length() {
        let input = vec!["a".to_string(), "ab".to_string(), "abc".to_string()];
        let result = filter_by_length(input, 2);
        assert_eq!(result, vec!["ab", "abc"]);
    }
}
use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T> {
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_duplicates(&mut self) -> Vec<T>
    where
        T: Eq + std::hash::Hash + Clone,
    {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for item in self.data.drain(..) {
            if seen.insert(item.clone()) {
                result.push(item);
            }
        }

        self.data = result.clone();
        result
    }

    pub fn filter<F>(&mut self, predicate: F) -> Vec<T>
    where
        F: Fn(&T) -> bool,
    {
        let mut result = Vec::new();
        let mut filtered = Vec::new();

        for item in self.data.drain(..) {
            if predicate(&item) {
                result.push(item.clone());
                filtered.push(item);
            }
        }

        self.data = filtered;
        result
    }

    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let mut cleaner = DataCleaner::new(vec![1, 2, 2, 3, 4, 4, 5]);
        let result = cleaner.remove_duplicates();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
        assert_eq!(cleaner.get_data(), &vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_filter() {
        let mut cleaner = DataCleaner::new(vec![1, 2, 3, 4, 5, 6]);
        let result = cleaner.filter(|&x| x % 2 == 0);
        assert_eq!(result, vec![2, 4, 6]);
        assert_eq!(cleaner.get_data(), &vec![2, 4, 6]);
    }
}use std::collections::HashSet;

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

    pub fn deduplicate_rows(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| {
            let row_string: String = row
                .iter()
                .map(|cell| cell.as_ref().unwrap_or(&"NULL".to_string()))
                .collect::<Vec<_>>()
                .join("|");
            seen.insert(row_string)
        });
    }

    pub fn get_clean_data(&self) -> &Vec<Vec<Option<String>>> {
        &self.data
    }

    pub fn count_rows(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_operations() {
        let mut cleaner = DataCleaner::new(vec![
            vec![Some("A".to_string()), Some("1".to_string())],
            vec![Some("B".to_string()), None],
            vec![Some("A".to_string()), Some("1".to_string())],
        ]);

        assert_eq!(cleaner.count_rows(), 3);
        
        cleaner.remove_null_rows();
        assert_eq!(cleaner.count_rows(), 2);
        
        cleaner.deduplicate_rows();
        assert_eq!(cleaner.count_rows(), 1);
        
        let clean_data = cleaner.get_clean_data();
        assert_eq!(clean_data[0][0], Some("A".to_string()));
        assert_eq!(clean_data[0][1], Some("1".to_string()));
    }
}