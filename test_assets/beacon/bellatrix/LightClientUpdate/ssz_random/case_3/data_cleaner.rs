
fn deduplicate_and_sort(data: &[i32]) -> Vec<i32> {
    let mut unique: Vec<i32> = Vec::new();
    
    for &value in data {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    
    unique.sort();
    unique
}

fn filter_positive(data: &[i32]) -> Vec<i32> {
    data.iter()
        .filter(|&&x| x > 0)
        .cloned()
        .collect()
}

pub fn process_data(data: &[i32]) -> Vec<i32> {
    let filtered = filter_positive(data);
    deduplicate_and_sort(&filtered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data() {
        let input = vec![5, -3, 2, 5, 0, 8, -1, 2];
        let result = process_data(&input);
        assert_eq!(result, vec![2, 5, 8]);
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<i32> = vec![];
        let result = process_data(&input);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_all_negative() {
        let input = vec![-5, -10, -1];
        let result = process_data(&input);
        assert_eq!(result, vec![]);
    }
}