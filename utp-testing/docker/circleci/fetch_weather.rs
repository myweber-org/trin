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
    #[error("City not found")]
    CityNotFound,
    #[error("Cache expired")]
    CacheExpired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: u8,
    pub description: String,
    pub wind_speed: f64,
    pub timestamp: SystemTime,
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: HashMap<String, (WeatherData, SystemTime)>,
    cache_duration: Duration,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        WeatherFetcher {
            api_key,
            base_url: "https://api.openweathermap.org/data/2.5/weather".to_string(),
            cache: HashMap::new(),
            cache_duration: Duration::from_secs(300),
        }
    }

    pub async fn fetch_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
        if let Some((cached_data, timestamp)) = self.cache.get(city) {
            if timestamp.elapsed().unwrap_or(Duration::MAX) < self.cache_duration {
                return Ok(cached_data.clone());
            }
        }

        let url = format!(
            "{}?q={}&appid={}&units=metric",
            self.base_url, city, self.api_key
        );

        let response = reqwest::get(&url).await?;
        
        if response.status().is_client_error() {
            return match response.status().as_u16() {
                401 => Err(WeatherError::InvalidApiKey),
                404 => Err(WeatherError::CityNotFound),
                _ => Err(WeatherError::RequestFailed(
                    reqwest::Error::from(response.status())
                )),
            };
        }

        let json: serde_json::Value = response.json().await?;
        
        let weather_data = WeatherData {
            temperature: json["main"]["temp"].as_f64().unwrap_or(0.0),
            humidity: json["main"]["humidity"].as_u64().unwrap_or(0) as u8,
            description: json["weather"][0]["description"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            wind_speed: json["wind"]["speed"].as_f64().unwrap_or(0.0),
            timestamp: SystemTime::now(),
        };

        self.cache.insert(city.to_string(), (weather_data.clone(), SystemTime::now()));
        
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
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/data/2.5/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "main": {"temp": 15.5, "humidity": 65},
                "weather": [{"description": "clear sky"}],
                "wind": {"speed": 3.2}
            }"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.base_url = server_url() + "/data/2.5/weather";
        
        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.humidity, 65);
        assert_eq!(weather.description, "clear sky");
        assert_eq!(weather.wind_speed, 3.2);
    }

    #[tokio::test]
    async fn test_fetch_weather_city_not_found() {
        let _m = mock("GET", "/data/2.5/weather?q=InvalidCity&appid=test_key&units=metric")
            .with_status(404)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.base_url = server_url() + "/data/2.5/weather";
        
        let result = fetcher.fetch_weather("InvalidCity").await;
        assert!(matches!(result, Err(WeatherError::CityNotFound)));
    }
}