use std::collections::HashSet;

pub fn clean_string_list(input: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    input
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| {
            let trimmed = s.trim().to_string();
            if seen.insert(trimmed.clone()) {
                Some(trimmed)
            } else {
                None
            }
        })
        .collect()
}