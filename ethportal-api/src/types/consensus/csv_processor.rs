use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn new(id: u32, name: &str, category: &str, value: f64, active: bool) -> Self {
        Record {
            id,
            name: name.to_string(),
            category: category.to_string(),
            value,
            active,
        }
    }

    fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

fn process_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    for result in csv_reader.deserialize() {
        let mut record: Record = result?;
        
        if record.value >= min_value && record.active {
            record.transform_value(1.15);
            csv_writer.serialize(&record)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}

fn generate_sample_data() -> Vec<Record> {
    vec![
        Record::new(1, "Alpha", "A", 25.5, true),
        Record::new(2, "Beta", "B", 12.0, false),
        Record::new(3, "Gamma", "A", 30.0, true),
        Record::new(4, "Delta", "C", 8.5, true),
        Record::new(5, "Epsilon", "B", 42.0, true),
    ]
}

fn main() -> Result<(), Box<dyn Error>> {
    let sample_records = generate_sample_data();
    
    let temp_input = "temp_input.csv";
    let temp_output = "processed_data.csv";
    
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(File::create(temp_input)?);
    
    for record in sample_records {
        writer.serialize(&record)?;
    }
    writer.flush()?;

    process_csv(temp_input, temp_output, 20.0)?;
    
    std::fs::remove_file(temp_input)?;
    
    println!("Processing completed. Results saved to {}", temp_output);
    Ok(())
}