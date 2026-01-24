use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn load_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.active)
        .collect()
}

fn calculate_category_averages(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut category_totals: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_totals
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_totals
        .into_iter()
        .map(|(category, (total, count))| (category, total / count as f64))
        .collect()
}

fn save_processed_data(
    records: &[&Record],
    averages: &[(String, f64)],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(output_path)?;

    writer.write_record(&["ID", "Name", "Category", "Value", "Status"])?;
    for record in records {
        writer.serialize((
            record.id,
            &record.name,
            &record.category,
            record.value,
            "Active",
        ))?;
    }

    writer.write_record(&[])?;
    writer.write_record(&["Category", "Average Value"])?;
    for (category, avg) in averages {
        writer.serialize((category, avg))?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let all_records = load_csv_data(input_path)?;
    let active_records = filter_active_records(&all_records);
    let category_averages = calculate_category_averages(&all_records);

    save_processed_data(&active_records, &category_averages, output_path)?;

    println!("Processed {} records", all_records.len());
    println!("Found {} active records", active_records.len());
    println!("Calculated averages for {} categories", category_averages.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record {
                id: 1,
                name: "Test1".to_string(),
                category: "A".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                category: "B".to_string(),
                value: 20.0,
                active: false,
            },
        ];

        let active = filter_active_records(&records);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }

    #[test]
    fn test_calculate_category_averages() {
        let records = vec![
            Record {
                id: 1,
                name: "Test1".to_string(),
                category: "A".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                category: "A".to_string(),
                value: 20.0,
                active: true,
            },
        ];

        let averages = calculate_category_averages(&records);
        assert_eq!(averages.len(), 1);
        assert_eq!(averages[0].0, "A");
        assert_eq!(averages[0].1, 15.0);
    }

    #[test]
    fn test_full_processing() -> Result<(), Box<dyn Error>> {
        let input_data = "id,name,category,value,active\n1,Item1,CategoryA,10.5,true\n2,Item2,CategoryB,15.0,false\n";
        
        let mut input_file = NamedTempFile::new()?;
        write!(input_file, "{}", input_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        process_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        )?;

        Ok(())
    }
}