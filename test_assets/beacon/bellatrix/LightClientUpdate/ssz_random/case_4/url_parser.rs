
use std::collections::HashMap;

pub struct QueryParser;

impl QueryParser {
    pub fn parse(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }
        
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(
                    key.to_string(),
                    urlencoding::decode(value)
                        .unwrap_or_else(|_| value.into())
                        .to_string()
                );
            }
        }
        
        params
    }
    
    pub fn get_param(query: &str, key: &str) -> Option<String> {
        Self::parse(query).get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_single_param() {
        let params = QueryParser::parse("name=john");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
    }
    
    #[test]
    fn test_parse_multiple_params() {
        let params = QueryParser::parse("name=john&age=25&city=new+york");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"25".to_string()));
        assert_eq!(params.get("city"), Some(&"new york".to_string()));
    }
    
    #[test]
    fn test_get_specific_param() {
        let value = QueryParser::get_param("name=alice&role=admin", "role");
        assert_eq!(value, Some("admin".to_string()));
    }
}