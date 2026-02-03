
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(input_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(h)) => h,
        Some(Err(e)) => return Err(Box::new(e)),
        None => return Err("Empty file".into()),
    };
    
    let mut seen = HashSet::new();
    let mut unique_lines = Vec::new();
    
    for line_result in lines {
        let line = line_result?;
        if !seen.contains(&line) {
            seen.insert(line.clone());
            unique_lines.push(line);
        }
    }
    
    let mut output_file = File::create(output_path)?;
    writeln!(output_file, "{}", header)?;
    
    for line in unique_lines {
        writeln!(output_file, "{}", line)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    
    #[test]
    fn test_remove_duplicates() {
        let test_input = "id,name,value\n1,test,100\n2,test,200\n1,test,100\n3,other,300";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";
        
        let mut input_file = File::create(input_path).unwrap();
        write!(input_file, "{}", test_input).unwrap();
        
        remove_duplicates(input_path, output_path).unwrap();
        
        let mut output_file = File::open(output_path).unwrap();
        let mut content = String::new();
        output_file.read_to_string(&mut content).unwrap();
        
        let expected = "id,name,value\n1,test,100\n2,test,200\n3,other,300\n";
        assert_eq!(content, expected);
        
        std::fs::remove_file(input_path).unwrap();
        std::fs::remove_file(output_path).unwrap();
    }
}use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

pub fn process_stream() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    let mut buffer = String::new();

    for line in stdin.lock().lines() {
        buffer.push_str(&line?);
        buffer.push('\n');
    }

    let cleaned = clean_data(&buffer);
    output.write_all(cleaned.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = "banana\napple\ncherry\napple\nbanana";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(clean_data(input), expected);
    }
}