use reqwest::Client;
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
    #[error("API key not configured")]
    MissingApiKey,
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: MainData,
    weather: Vec<WeatherCondition>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Debug, Deserialize)]
struct WeatherCondition {
    main: String,
    description: String,
}

pub struct WeatherFetcher {
    client: Client,
    api_key: String,
    cache: HashMap<String, (WeatherData, Instant)>,
    cache_ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub city: String,
    pub temperature: f64,
    pub humidity: u8,
    pub pressure: u16,
    pub condition: String,
    pub description: String,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(300),
        }
    }

    pub async fn fetch_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
        let cache_key = city.to_lowercase();

        if let Some((data, timestamp)) = self.cache.get(&cache_key) {
            if timestamp.elapsed() < self.cache_ttl {
                return Ok(data.clone());
            }
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(WeatherError::InvalidResponse(
                format!("HTTP {}: {}", response.status(), response.text().await?)
            ));
        }

        let weather_response: WeatherResponse = response.json().await?;
        
        let weather_data = WeatherData {
            city: weather_response.name,
            temperature: weather_response.main.temp,
            humidity: weather_response.main.humidity,
            pressure: weather_response.main.pressure,
            condition: weather_response.weather[0].main.clone(),
            description: weather_response.weather[0].description.clone(),
        };

        self.cache.insert(cache_key, (weather_data.clone(), Instant::now()));
        
        Ok(weather_data)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn set_cache_ttl(&mut self, ttl: Duration) {
        self.cache_ttl = ttl;
    }
}