use std::collections::HashMap;

pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    
    if query.is_empty() {
        return params;
    }
    
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            let decoded_key = urlencoding::decode(key).unwrap_or_else(|_| key.into());
            let decoded_value = urlencoding::decode(value).unwrap_or_else(|_| value.into());
            params.insert(decoded_key.into_owned(), decoded_value.into_owned());
        }
    }
    
    params
}

pub fn build_query_string(params: &HashMap<String, String>) -> String {
    let mut pairs: Vec<String> = Vec::new();
    
    for (key, value) in params {
        let encoded_key = urlencoding::encode(key);
        let encoded_value = urlencoding::encode(value);
        pairs.push(format!("{}={}", encoded_key, encoded_value));
    }
    
    pairs.join("&")
}