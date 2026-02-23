use regex::Regex;
use std::collections::HashSet;

pub fn sanitize_input(input: &str) -> String {
    let trimmed = input.trim();
    
    let re = Regex::new(r"[^\w\s\-.,!?;:]").unwrap();
    let cleaned = re.replace_all(trimmed, "").to_string();
    
    cleaned
}

pub fn normalize_whitespace(text: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(text, " ").to_string()
}

pub fn remove_duplicate_lines(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    
    let mut result = Vec::new();
    for line in lines {
        if unique_lines.contains(line) {
            result.push(line);
        }
    }
    
    result.join("\n")
}

pub fn clean_data_pipeline(input: &str) -> String {
    let step1 = sanitize_input(input);
    let step2 = normalize_whitespace(&step1);
    let step3 = remove_duplicate_lines(&step2);
    
    step3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_input() {
        let input = "Hello@World#123!";
        let expected = "HelloWorld123!";
        assert_eq!(sanitize_input(input), expected);
    }

    #[test]
    fn test_normalize_whitespace() {
        let input = "Hello    World\n\nTest";
        let expected = "Hello World Test";
        assert_eq!(normalize_whitespace(input), expected);
    }

    #[test]
    fn test_clean_data_pipeline() {
        let input = "Line1\nLine2\nLine1\n\n   Extra   spaces   ";
        let expected = "Line1\nLine2\nExtra spaces";
        assert_eq!(clean_data_pipeline(input), expected);
    }
}