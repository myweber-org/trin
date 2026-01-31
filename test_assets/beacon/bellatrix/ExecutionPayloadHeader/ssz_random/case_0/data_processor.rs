
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }

        if self.values.is_empty() {
            return Err("Empty values vector".into());
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".into());
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();

    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.normalize();
        processed.push(processed_record);
    }

    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();

    if records.is_empty() {
        return stats;
    }

    let value_count = records[0].values.len();
    let mut sums = vec![0.0; value_count];
    let mut squares = vec![0.0; value_count];

    for record in records {
        for (i, &value) in record.values.iter().enumerate() {
            sums[i] += value;
            squares[i] += value * value;
        }
    }

    let n = records.len() as f64;
    for i in 0..value_count {
        let mean = sums[i] / n;
        let variance = (squares[i] / n) - (mean * mean);
        let std_dev = variance.sqrt();

        stats.insert(format!("mean_{}", i), mean);
        stats.insert(format!("std_dev_{}", i), std_dev);
    }

    stats
}