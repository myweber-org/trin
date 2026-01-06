use std::collections::HashMap;

pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    
    if query.is_empty() {
        return params;
    }
    
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let Some(key) = parts.next() {
            let value = parts.next().unwrap_or("");
            params.insert(key.to_string(), value.to_string());
        }
    }
    
    params
}

pub fn extract_query_from_url(url: &str) -> Option<String> {
    url.split('?')
        .nth(1)
        .map(|s| s.split('#').next().unwrap_or(s).to_string())
}

pub fn parse_url_query(url: &str) -> HashMap<String, String> {
    extract_query_from_url(url)
        .map(|query| parse_query_string(&query))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=new+york";
        let params = parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"new+york".to_string()));
    }

    #[test]
    fn test_extract_query_from_url() {
        let url = "https://example.com/path?query=test&page=1#section";
        let query = extract_query_from_url(url).unwrap();
        assert_eq!(query, "query=test&page=1");
    }

    #[test]
    fn test_parse_url_query() {
        let url = "https://api.example.com/search?q=rust&limit=10";
        let params = parse_url_query(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("limit"), Some(&"10".to_string()));
    }
}