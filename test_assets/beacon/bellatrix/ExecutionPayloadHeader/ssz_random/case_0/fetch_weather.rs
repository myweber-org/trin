use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Invalid API response")]
    InvalidResponse,
    #[error("Location not found")]
    LocationNotFound,
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: MainData,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    humidity: u8,
}

pub struct WeatherCache {
    cache: HashMap<String, (WeatherData, Instant)>,
    ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub location: String,
    pub temperature: f64,
    pub humidity: u8,
    pub timestamp: Instant,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn get_weather(
        &mut self,
        api_key: &str,
        city: &str,
    ) -> Result<WeatherData, WeatherError> {
        let cache_key = city.to_lowercase();

        if let Some((data, timestamp)) = self.cache.get(&cache_key) {
            if timestamp.elapsed() < self.ttl {
                return Ok(data.clone());
            }
        }

        let weather = Self::fetch_from_api(api_key, city).await?;
        self.cache.insert(cache_key, (weather.clone(), weather.timestamp));
        Ok(weather)
    }

    async fn fetch_from_api(api_key: &str, city: &str) -> Result<WeatherData, WeatherError> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, api_key
        );

        let response = reqwest::get(&url).await?;
        
        if response.status().is_client_error() {
            return Err(WeatherError::LocationNotFound);
        }

        let weather_response: WeatherResponse = response.json().await
            .map_err(|_| WeatherError::InvalidResponse)?;

        Ok(WeatherData {
            location: weather_response.name,
            temperature: weather_response.main.temp,
            humidity: weather_response.main.humidity,
            timestamp: Instant::now(),
        })
    }

    pub fn clear_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, (_, timestamp)| {
            timestamp.elapsed() < self.ttl
        });
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_weather_fetch() {
        let mut server = mockito::Server::new();
        let mock = server.mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();

        let _guard = mockito::server_url(&server);
        
        let mut cache = WeatherCache::new(300);
        let result = cache.get_weather("test_key", "London").await;
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.location, "London");
        assert_eq!(data.temperature, 15.5);
        assert_eq!(data.humidity, 65);
    }
}