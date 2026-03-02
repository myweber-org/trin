
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(String),
    #[error("API response parsing failed: {0}")]
    ParseError(String),
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub description: String,
    pub timestamp: Instant,
}

pub struct WeatherCache {
    cache: HashMap<String, (WeatherData, Instant)>,
    ttl: Duration,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, location: &str) -> Option<&WeatherData> {
        self.cache.get(location).and_then(|(data, timestamp)| {
            if timestamp.elapsed() < self.ttl {
                Some(data)
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, location: String, data: WeatherData) {
        self.cache.insert(location, (data, Instant::now()));
    }

    pub fn clear_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, (_, timestamp)| {
            timestamp.elapsed() < self.ttl
        });
    }
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: WeatherCache,
}

impl WeatherFetcher {
    pub fn new(api_key: String, cache_ttl_seconds: u64) -> Self {
        WeatherFetcher {
            api_key,
            base_url: "https://api.weather.example.com".to_string(),
            cache: WeatherCache::new(cache_ttl_seconds),
        }
    }

    pub async fn fetch_weather(&mut self, location: &str) -> Result<WeatherData, WeatherError> {
        if let Some(cached) = self.cache.get(location) {
            return Ok(cached.clone());
        }

        let url = format!("{}/weather?location={}&api_key={}", 
                         self.base_url, location, self.api_key);
        
        let response = reqwest::get(&url)
            .await
            .map_err(|e| WeatherError::NetworkError(e.to_string()))?;

        if response.status() == 401 {
            return Err(WeatherError::InvalidApiKey);
        }

        if response.status() == 429 {
            return Err(WeatherError::RateLimitExceeded);
        }

        let json: serde_json::Value = response.json()
            .await
            .map_err(|e| WeatherError::ParseError(e.to_string()))?;

        let weather_data = WeatherData {
            temperature: json["main"]["temp"].as_f64()
                .ok_or_else(|| WeatherError::ParseError("Invalid temperature".to_string()))?,
            humidity: json["main"]["humidity"].as_f64()
                .ok_or_else(|| WeatherError::ParseError("Invalid humidity".to_string()))?,
            pressure: json["main"]["pressure"].as_f64()
                .ok_or_else(|| WeatherError::ParseError("Invalid pressure".to_string()))?,
            description: json["weather"][0]["description"].as_str()
                .unwrap_or("unknown").to_string(),
            timestamp: Instant::now(),
        };

        self.cache.insert(location.to_string(), weather_data.clone());
        Ok(weather_data)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear_expired();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_retrieve() {
        let mut cache = WeatherCache::new(300);
        let test_data = WeatherData {
            temperature: 25.0,
            humidity: 65.0,
            pressure: 1013.0,
            description: "clear sky".to_string(),
            timestamp: Instant::now(),
        };

        cache.insert("London".to_string(), test_data.clone());
        
        let retrieved = cache.get("London");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().temperature, 25.0);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = WeatherCache::new(1);
        let test_data = WeatherData {
            temperature: 25.0,
            humidity: 65.0,
            pressure: 1013.0,
            description: "clear sky".to_string(),
            timestamp: Instant::now(),
        };

        cache.insert("London".to_string(), test_data);
        std::thread::sleep(Duration::from_secs(2));
        
        let retrieved = cache.get("London");
        assert!(retrieved.is_none());
    }
}