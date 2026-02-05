use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub fn read_csv<P: AsRef<Path>>(file_path: P) -> io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let columns: Vec<&str> = line.split(',').collect();
        println!("{:?}", columns);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let result = read_csv(temp_file.path());
        assert!(result.is_ok());
    }
}