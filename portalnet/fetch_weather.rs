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

struct CacheEntry {
    data: WeatherDataCached,
    timestamp: SystemTime,
}

struct WeatherDataCached {
    temperature: f64,
    humidity: u8,
    description: String,
    location: String,
}

pub struct WeatherFetcher {
    api_key: String,
    cache: HashMap<String, CacheEntry>,
    cache_duration: Duration,
    client: reqwest::Client,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        WeatherFetcher {
            api_key,
            cache: HashMap::new(),
            cache_duration: Duration::from_secs(300),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_weather(&mut self, city: &str) -> Result<WeatherDataCached, WeatherError> {
        let cache_key = city.to_lowercase();

        if let Some(entry) = self.cache.get(&cache_key) {
            if entry.timestamp.elapsed().unwrap_or_default() < self.cache_duration {
                return Ok(entry.data.clone());
            }
        }

        let weather_data = self.fetch_remote(city).await?;
        let cached_data = WeatherDataCached {
            temperature: weather_data.main.temp,
            humidity: weather_data.main.humidity,
            description: weather_data.weather.first()
                .map(|w| w.description.clone())
                .unwrap_or_default(),
            location: weather_data.name,
        };

        self.cache.insert(
            cache_key,
            CacheEntry {
                data: cached_data.clone(),
                timestamp: SystemTime::now(),
            },
        );

        Ok(cached_data)
    }

    async fn fetch_remote(&self, city: &str) -> Result<WeatherResponse, WeatherError> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let weather: WeatherResponse = response.json().await?;
            Ok(weather)
        } else if response.status().as_u16() == 404 {
            Err(WeatherError::LocationNotFound)
        } else {
            Err(WeatherError::InvalidResponse)
        }
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn set_cache_duration(&mut self, duration: Duration) {
        self.cache_duration = duration;
    }
}

impl Clone for WeatherDataCached {
    fn clone(&self) -> Self {
        WeatherDataCached {
            temperature: self.temperature,
            humidity: self.humidity,
            description: self.description.clone(),
            location: self.location.clone(),
        }
    }
}