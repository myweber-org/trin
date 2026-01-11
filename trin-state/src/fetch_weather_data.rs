use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Location not found")]
    LocationNotFound,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub wind_speed: f64,
    pub description: String,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: WeatherData,
    expires_at: SystemTime,
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: HashMap<String, CacheEntry>,
    cache_duration: Duration,
    client: reqwest::Client,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.weatherservice.com/v1".to_string(),
            cache: HashMap::new(),
            cache_duration: Duration::from_secs(300),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_weather(&mut self, location: &str) -> Result<WeatherData, WeatherError> {
        let cache_key = location.to_lowercase();

        if let Some(entry) = self.cache.get(&cache_key) {
            if SystemTime::now() < entry.expires_at {
                return Ok(entry.data.clone());
            }
        }

        let url = format!("{}/current?location={}&apikey={}", 
            self.base_url, location, self.api_key);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return match response.status().as_u16() {
                401 => Err(WeatherError::InvalidApiKey),
                404 => Err(WeatherError::LocationNotFound),
                429 => Err(WeatherError::RateLimitExceeded),
                _ => Err(WeatherError::RequestFailed(response.error_for_status().unwrap_err())),
            };
        }

        let weather_data: WeatherData = response.json().await?;
        let cache_entry = CacheEntry {
            data: weather_data.clone(),
            expires_at: SystemTime::now() + self.cache_duration,
        };

        self.cache.insert(cache_key, cache_entry);
        Ok(weather_data)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn set_cache_duration(&mut self, duration: Duration) {
        self.cache_duration = duration;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_successful_weather_fetch() {
        let _m = mock("GET", "/v1/current?location=london&apikey=test_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"temperature":15.5,"humidity":65.0,"pressure":1013.0,"wind_speed":5.2,"description":"Partly cloudy"}"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.base_url = server_url();

        let result = fetcher.get_weather("london").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.description, "Partly cloudy");
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.set_cache_duration(Duration::from_secs(10));

        let weather_data = WeatherData {
            temperature: 20.0,
            humidity: 50.0,
            pressure: 1015.0,
            wind_speed: 3.0,
            description: "Sunny".to_string(),
            timestamp: SystemTime::now(),
        };

        let cache_entry = CacheEntry {
            data: weather_data.clone(),
            expires_at: SystemTime::now() + Duration::from_secs(10),
        };

        fetcher.cache.insert("paris".to_string(), cache_entry);

        let result = fetcher.get_weather("Paris").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().temperature, 20.0);
    }
}