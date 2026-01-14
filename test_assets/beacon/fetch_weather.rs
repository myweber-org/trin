use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Invalid API response")]
    InvalidResponse,
    #[error("Cache expired")]
    CacheExpired,
}

#[derive(Deserialize, Debug, Clone)]
struct WeatherData {
    main: MainData,
    weather: Vec<WeatherInfo>,
    name: String,
}

#[derive(Deserialize, Debug, Clone)]
struct MainData {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Deserialize, Debug, Clone)]
struct WeatherInfo {
    description: String,
    icon: String,
}

struct WeatherCache {
    data: HashMap<String, (WeatherData, SystemTime)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&self, city: &str) -> Option<WeatherData> {
        self.data.get(city).and_then(|(data, timestamp)| {
            if timestamp.elapsed().ok()? < self.ttl {
                Some(data.clone())
            } else {
                None
            }
        })
    }

    fn insert(&mut self, city: String, data: WeatherData) {
        self.data.insert(city, (data, SystemTime::now()));
    }
}

pub struct WeatherFetcher {
    api_key: String,
    client: reqwest::Client,
    cache: WeatherCache,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        WeatherFetcher {
            api_key,
            client: reqwest::Client::new(),
            cache: WeatherCache::new(300),
        }
    }

    pub async fn fetch_weather(&mut self, city: &str) -> Result<String, WeatherError> {
        if let Some(cached) = self.cache.get(city) {
            return Ok(format!(
                "{}: {:.1}°C, {}",
                cached.name, cached.main.temp, cached.weather[0].description
            ));
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(WeatherError::InvalidResponse);
        }

        let weather_data: WeatherData = response.json().await?;
        self.cache.insert(city.to_string(), weather_data.clone());

        Ok(format!(
            "{}: {:.1}°C, {}",
            weather_data.name, weather_data.main.temp, weather_data.weather[0].description
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "name": "London",
                "main": {"temp": 15.5, "humidity": 65, "pressure": 1013},
                "weather": [{"description": "clear sky", "icon": "01d"}]
            }"#)
            .create_async()
            .await;

        let api_key = "test_key".to_string();
        let mut fetcher = WeatherFetcher::new(api_key);
        
        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("London"));

        mock.assert_async().await;
    }
}