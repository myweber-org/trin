use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<Vec<T>>,
}

impl<T> DataCleaner<T>
where
    T: Clone + PartialEq + Eq + std::hash::Hash,
{
    pub fn new(data: Vec<Vec<T>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self, null_value: &T) {
        self.data.retain(|row| !row.iter().any(|cell| cell == null_value));
    }

    pub fn deduplicate_rows(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| seen.insert(row.clone()));
    }

    pub fn get_data(&self) -> &Vec<Vec<T>> {
        &self.data
    }

    pub fn into_data(self) -> Vec<Vec<T>> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_operations() {
        let mut data = vec![
            vec!["A", "B", "C"],
            vec!["null", "E", "F"],
            vec!["A", "B", "C"],
            vec!["G", "H", "I"],
        ];

        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_rows(&"null");
        cleaner.deduplicate_rows();

        let result = cleaner.into_data();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["A", "B", "C"]);
        assert_eq!(result[1], vec!["G", "H", "I"]);
    }
}