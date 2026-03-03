use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn clean_csv_duplicates(input_path: &str, output_path: &str) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("Empty or invalid CSV file".into()),
    };

    let mut seen = HashSet::new();
    let mut unique_lines = Vec::new();
    let mut duplicate_count = 0;

    for line_result in lines {
        let line = line_result?;
        if seen.insert(line.clone()) {
            unique_lines.push(line);
        } else {
            duplicate_count += 1;
        }
    }

    let mut output_file = File::create(Path::new(output_path))?;
    writeln!(output_file, "{}", header)?;
    for line in unique_lines {
        writeln!(output_file, "{}", line)?;
    }

    Ok(duplicate_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_duplicate_removal() {
        let input_content = "id,name,value\n1,test,100\n2,other,200\n1,test,100\n3,third,300";
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let duplicates = clean_csv_duplicates(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        assert_eq!(duplicates, 1);
        
        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
            
        let expected = "id,name,value\n1,test,100\n2,other,200\n3,third,300\n";
        assert_eq!(output_content, expected);
    }
}