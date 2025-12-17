
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue(f64),
    InvalidCategory(String),
    EmptyData,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            DataError::InvalidCategory(c) => write!(f, "Invalid category: {}", c),
            DataError::EmptyData => write!(f, "Empty data provided"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        Self::validate_record(&record)?;
        self.records.push(record);
        Ok(())
    }

    pub fn process_records(&self) -> Result<Vec<DataRecord>, DataError> {
        if self.records.is_empty() {
            return Err(DataError::EmptyData);
        }

        let mut processed = Vec::with_capacity(self.records.len());
        for record in &self.records {
            let transformed = Self::transform_record(record)?;
            processed.push(transformed);
        }

        Ok(processed)
    }

    fn validate_record(record: &DataRecord) -> Result<(), DataError> {
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(DataError::InvalidValue(record.value));
        }

        if record.category.is_empty() || record.category.len() > 50 {
            return Err(DataError::InvalidCategory(record.category.clone()));
        }

        Ok(())
    }

    fn transform_record(record: &DataRecord) -> Result<DataRecord, DataError> {
        let normalized_value = if record.value > 500.0 {
            record.value / 2.0
        } else {
            record.value * 1.5
        };

        let normalized_category = record.category.to_uppercase();

        Ok(DataRecord {
            id: record.id,
            value: normalized_value,
            category: normalized_category,
        })
    }

    pub fn get_statistics(&self) -> Option<(f64, f64, f64)> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let average = sum / count;

        let min = self
            .records
            .iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);
        let max = self
            .records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        Some((average, min, max))
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}