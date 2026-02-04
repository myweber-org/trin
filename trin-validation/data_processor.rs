use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataPoint {
    timestamp: i64,
    value: f64,
    category: String,
}

impl DataPoint {
    pub fn new(timestamp: i64, value: f64, category: &str) -> Self {
        DataPoint {
            timestamp,
            value,
            category: category.to_string(),
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn category(&self) -> &str {
        &self.category
    }
}

pub struct DataProcessor {
    data: Vec<DataPoint>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut rdr = csv::Reader::from_reader(reader);

        for result in rdr.records() {
            let record = result?;
            if record.len() >= 3 {
                let timestamp: i64 = record[0].parse()?;
                let value: f64 = record[1].parse()?;
                let category = record[2].to_string();
                self.data.push(DataPoint::new(timestamp, value, &category));
            }
        }
        Ok(())
    }

    pub fn add_data_point(&mut self, point: DataPoint) {
        self.data.push(point);
    }

    pub fn calculate_statistics(&self, category_filter: Option<&str>) -> (f64, f64, f64) {
        let filtered_data: Vec<&DataPoint> = match category_filter {
            Some(category) => self.data
                .iter()
                .filter(|point| point.category == category)
                .collect(),
            None => self.data.iter().collect(),
        };

        if filtered_data.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = filtered_data.iter().map(|p| p.value).sum();
        let count = filtered_data.len() as f64;
        let mean = sum / count;

        let variance: f64 = filtered_data
            .iter()
            .map(|p| (p.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_value_range(&self, min: f64, max: f64) -> Vec<DataPoint> {
        self.data
            .iter()
            .filter(|point| point.value >= min && point.value <= max)
            .cloned()
            .collect()
    }

    pub fn export_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut writer = csv::Writer::from_writer(BufWriter::new(file));

        writer.write_record(&["timestamp", "value", "category"])?;
        for point in &self.data {
            writer.write_record(&[
                point.timestamp.to_string(),
                point.value.to_string(),
                point.category.clone(),
            ])?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
    }

    pub fn clear_data(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.data_count(), 0);

        processor.add_data_point(DataPoint::new(1000, 42.5, "A"));
        processor.add_data_point(DataPoint::new(1001, 37.8, "B"));
        processor.add_data_point(DataPoint::new(1002, 45.2, "A"));

        assert_eq!(processor.data_count(), 3);

        let stats = processor.calculate_statistics(Some("A"));
        assert!((stats.0 - 43.85).abs() < 0.01);

        let filtered = processor.filter_by_value_range(40.0, 50.0);
        assert_eq!(filtered.len(), 2);

        let temp_file = NamedTempFile::new().unwrap();
        processor.export_to_csv(temp_file.path()).unwrap();
        
        processor.clear_data();
        assert_eq!(processor.data_count(), 0);
    }
}