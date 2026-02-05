use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Invalid API response")]
    InvalidResponse,
    #[error("Location not found")]
    LocationNotFound,
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: MainData,
    weather: Vec<WeatherInfo>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Debug, Deserialize)]
struct WeatherInfo {
    description: String,
    icon: String,
}

#[derive(Debug)]
struct CachedWeather {
    data: WeatherData,
    timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub location: String,
    pub temperature: f64,
    pub humidity: u8,
    pub pressure: u16,
    pub description: String,
    pub icon_code: String,
}

pub struct WeatherFetcher {
    client: Client,
    api_key: String,
    cache: HashMap<String, CachedWeather>,
    cache_duration: Duration,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            cache: HashMap::new(),
            cache_duration: Duration::from_secs(300),
        }
    }

    pub async fn get_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
        let cache_key = city.to_lowercase();

        if let Some(cached) = self.cache.get(&cache_key) {
            if cached.timestamp.elapsed().unwrap_or(self.cache_duration) < self.cache_duration {
                return Ok(cached.data.clone());
            }
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;

        if response.status().is_client_error() {
            return Err(WeatherError::LocationNotFound);
        }

        let weather_response: WeatherResponse = response.json().await?;

        let weather_data = WeatherData {
            location: weather_response.name,
            temperature: weather_response.main.temp,
            humidity: weather_response.main.humidity,
            pressure: weather_response.main.pressure,
            description: weather_response.weather[0].description.clone(),
            icon_code: weather_response.weather[0].icon.clone(),
        };

        self.cache.insert(
            cache_key,
            CachedWeather {
                data: weather_data.clone(),
                timestamp: SystemTime::now(),
            },
        );

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
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_weather_fetching() {
        let _m = mock("GET", "/data/2.5/weather")
            .match_query(Matcher::UrlEncoded("q".into(), "London".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "name": "London",
                "main": {"temp": 15.5, "humidity": 65, "pressure": 1013},
                "weather": [{"description": "clear sky", "icon": "01d"}]
            }"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.set_cache_duration(Duration::from_secs(0));

        let result = fetcher.get_weather("London").await;
        assert!(result.is_ok());

        let weather = result.unwrap();
        assert_eq!(weather.location, "London");
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.description, "clear sky");
    }
}