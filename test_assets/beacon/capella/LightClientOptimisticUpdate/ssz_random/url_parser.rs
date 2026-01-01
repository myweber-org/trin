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
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                if !key.is_empty() {
                    params.insert(
                        key.to_string(),
                        urlencoding::decode(value)
                            .unwrap_or_else(|_| value.into())
                            .to_string()
                    );
                }
            }
        }
        
        params
    }
    
    pub fn build(params: &HashMap<String, String>) -> String {
        let mut pairs: Vec<String> = Vec::new();
        
        for (key, value) in params {
            let encoded_key = urlencoding::encode(key);
            let encoded_value = urlencoding::encode(value);
            pairs.push(format!("{}={}", encoded_key, encoded_value));
        }
        
        pairs.join("&")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = QueryParser::parse("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_single_param() {
        let result = QueryParser::parse("name=john");
        assert_eq!(result.get("name"), Some(&"john".to_string()));
    }

    #[test]
    fn test_parse_multiple_params() {
        let result = QueryParser::parse("name=john&age=25&city=new+york");
        assert_eq!(result.get("name"), Some(&"john".to_string()));
        assert_eq!(result.get("age"), Some(&"25".to_string()));
        assert_eq!(result.get("city"), Some(&"new york".to_string()));
    }

    #[test]
    fn test_parse_encoded_values() {
        let result = QueryParser::parse("message=hello%20world%21");
        assert_eq!(result.get("message"), Some(&"hello world!".to_string()));
    }

    #[test]
    fn test_build_params() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), "john doe".to_string());
        params.insert("age".to_string(), "30".to_string());
        
        let query = QueryParser::build(&params);
        assert!(query.contains("name=john%20doe"));
        assert!(query.contains("age=30"));
    }

    #[test]
    fn test_roundtrip() {
        let original = "name=john&age=25&city=new+york";
        let parsed = QueryParser::parse(original);
        let rebuilt = QueryParser::build(&parsed);
        
        let parsed_again = QueryParser::parse(&rebuilt);
        assert_eq!(parsed, parsed_again);
    }
}