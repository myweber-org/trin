use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),
    #[error("Location not found")]
    LocationNotFound,
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: MainData,
    weather: Vec<WeatherData>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Debug, Deserialize)]
struct WeatherData {
    description: String,
    icon: String,
}

pub struct WeatherCache {
    cache: HashMap<String, (Instant, WeatherData)>,
    ttl: Duration,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, location: &str) -> Option<&WeatherData> {
        self.cache.get(location).and_then(|(timestamp, data)| {
            if timestamp.elapsed() < self.ttl {
                Some(data)
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, location: String, data: WeatherData) {
        self.cache.insert(location, (Instant::now(), data));
    }

    pub fn clear_expired(&mut self) {
        self.cache.retain(|_, (timestamp, _)| timestamp.elapsed() < self.ttl);
    }
}

pub struct WeatherFetcher {
    client: reqwest::Client,
    api_key: String,
    cache: WeatherCache,
}

impl WeatherFetcher {
    pub fn new(api_key: String, cache_ttl: u64) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            cache: WeatherCache::new(cache_ttl),
        }
    }

    pub async fn fetch_weather(&mut self, location: &str) -> Result<WeatherData, WeatherError> {
        self.cache.clear_expired();

        if let Some(cached) = self.cache.get(location) {
            return Ok(cached.clone());
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            location, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let weather_response: WeatherResponse = response.json().await?;
            
            if let Some(weather_data) = weather_response.weather.first() {
                let data = WeatherData {
                    description: weather_data.description.clone(),
                    icon: weather_data.icon.clone(),
                };
                
                self.cache.insert(location.to_string(), data.clone());
                Ok(data)
            } else {
                Err(WeatherError::InvalidResponse("No weather data found".to_string()))
            }
        } else if response.status() == 404 {
            Err(WeatherError::LocationNotFound)
        } else {
            Err(WeatherError::InvalidResponse(format!("HTTP {}", response.status())))
        }
    }

    pub fn get_cache_stats(&self) -> (usize, u64) {
        (self.cache.cache.len(), self.cache.ttl.as_secs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let mut server = mockito::Server::new();
        let mock = server.mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "weather": [{"description": "clear sky", "icon": "01d"}],
                "main": {"temp": 20.5, "humidity": 65, "pressure": 1013},
                "name": "London"
            }"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string(), 300);
        fetcher.client = reqwest::Client::new();
        
        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let weather_data = result.unwrap();
        assert_eq!(weather_data.description, "clear sky");
        assert_eq!(weather_data.icon, "01d");
        
        mock.assert();
    }

    #[test]
    fn test_cache_operations() {
        let mut cache = WeatherCache::new(60);
        let test_data = WeatherData {
            description: "test".to_string(),
            icon: "test".to_string(),
        };

        cache.insert("London".to_string(), test_data);
        assert!(cache.get("London").is_some());
        assert!(cache.get("Paris").is_none());
        
        cache.clear_expired();
        assert!(cache.get("London").is_some());
    }
}