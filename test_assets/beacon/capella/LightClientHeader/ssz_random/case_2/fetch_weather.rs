use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: Main,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Main {
    temp: f64,
    humidity: u8,
}

#[derive(Debug, Clone)]
struct CachedWeather {
    data: WeatherResponse,
    timestamp: Instant,
}

#[derive(Debug, Error)]
enum WeatherError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("City not found")]
    CityNotFound,
    #[error("Cache expired")]
    CacheExpired,
}

struct WeatherFetcher {
    client: Client,
    cache: Arc<RwLock<HashMap<String, CachedWeather>>>,
    ttl: Duration,
    api_key: String,
}

impl WeatherFetcher {
    fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(300),
            api_key,
        }
    }

    async fn fetch_weather(&self, city: &str) -> Result<WeatherResponse, WeatherError> {
        if let Some(cached) = self.get_cached(city) {
            return Ok(cached.data.clone());
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let weather: WeatherResponse = response.json().await?;
            self.cache_weather(city, &weather);
            Ok(weather)
        } else {
            Err(WeatherError::CityNotFound)
        }
    }

    fn get_cached(&self, city: &str) -> Option<CachedWeather> {
        let cache = self.cache.read().unwrap();
        cache.get(city).and_then(|cached| {
            if cached.timestamp.elapsed() < self.ttl {
                Some(cached.clone())
            } else {
                None
            }
        })
    }

    fn cache_weather(&self, city: &str, weather: &WeatherResponse) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(
            city.to_string(),
            CachedWeather {
                data: weather.clone(),
                timestamp: Instant::now(),
            },
        );
    }

    fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
}

#[tokio::main]
async fn main() -> Result<(), WeatherError> {
    let api_key = std::env::var("WEATHER_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
    let fetcher = WeatherFetcher::new(api_key);
    
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