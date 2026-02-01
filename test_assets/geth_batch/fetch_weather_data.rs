use std::collections::HashMap;
use std::time::{Duration, Instant};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

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

#[derive(Debug, Error)]
enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Invalid API response")]
    InvalidResponse,
}

struct WeatherCache {
    data: HashMap<String, (WeatherResponse, Instant)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        Self {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&self, city: &str) -> Option<&WeatherResponse> {
        self.data.get(city).and_then(|(response, timestamp)| {
            if timestamp.elapsed() < self.ttl {
                Some(response)
            } else {
                None
            }
        })
    }

    fn insert(&mut self, city: String, response: WeatherResponse) {
        self.data.insert(city, (response, Instant::now()));
    }
}

struct WeatherFetcher {
    client: Client,
    api_key: String,
    cache: WeatherCache,
}

impl WeatherFetcher {
    fn new(api_key: String, cache_ttl: u64) -> Self {
        Self {
            client: Client::new(),
            api_key,
            cache: WeatherCache::new(cache_ttl),
        }
    }

    async fn fetch_weather(&mut self, city: &str) -> Result<WeatherResponse, WeatherError> {
        if let Some(cached) = self.cache.get(city) {
            return Ok(cached.clone());
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(WeatherError::InvalidResponse);
        }

        let weather_data: WeatherResponse = response.json().await?;
        self.cache.insert(city.to_string(), weather_data.clone());
        
        Ok(weather_data)
    }
}

#[tokio::main]
async fn main() -> Result<(), WeatherError> {
    let api_key = std::env::var("WEATHER_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
    let mut fetcher = WeatherFetcher::new(api_key, 300);

    match fetcher.fetch_weather("London").await {
        Ok(weather) => {
            println!("Weather in {}: {:.1}°C, {}% humidity", 
                     weather.name, weather.main.temp, weather.main.humidity);
        }
        Err(e) => {
            eprintln!("Failed to fetch weather: {}", e);
        }
    }

    Ok(())
}