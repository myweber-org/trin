
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

impl CsvRecord {
    pub fn from_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 5 {
            return Err("Invalid CSV line format".into());
        }

        Ok(CsvRecord {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            category: parts[2].to_string(),
            value: parts[3].parse()?,
            active: parts[4].parse()?,
        })
    }

    pub fn to_line(&self) -> String {
        format!("{},{},{},{},{}", self.id, self.name, self.category, self.value, self.active)
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let record = CsvRecord::from_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn aggregate_by_category(&self) -> Vec<(String, f64, usize)> {
        use std::collections::HashMap;

        let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            let entry = aggregates
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        aggregates
            .into_iter()
            .map(|(category, (total, count))| (category, total, count))
            .collect()
    }

    pub fn save_to_file(&self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(filepath)?;
        writeln!(file, "# CSV Export")?;
        writeln!(file, "id,name,category,value,active")?;

        for record in &self.records {
            writeln!(file, "{}", record.to_line())?;
        }

        Ok(())
    }

    pub fn add_record(&mut self, record: CsvRecord) {
        self.records.push(record);
    }

    pub fn remove_record_by_id(&mut self, id: u32) -> Option<CsvRecord> {
        if let Some(pos) = self.records.iter().position(|r| r.id == id) {
            Some(self.records.remove(pos))
        } else {
            None
        }
    }

    pub fn get_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn get_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.get_total_value() / self.records.len() as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_parsing() {
        let line = "1,ProductA,Electronics,299.99,true";
        let record = CsvRecord::from_line(line).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "ProductA");
        assert_eq!(record.category, "Electronics");
        assert_eq!(record.value, 299.99);
        assert_eq!(record.active, true);
    }

    #[test]
    fn test_filter_active() {
        let mut processor = CsvProcessor::new();
        processor.add_record(CsvRecord {
            id: 1,
            name: "Item1".to_string(),
            category: "Test".to_string(),
            value: 10.0,
            active: true,
        });
        processor.add_record(CsvRecord {
            id: 2,
            name: "Item2".to_string(),
            category: "Test".to_string(),
            value: 20.0,
            active: false,
        });

        let active = processor.filter_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }
}