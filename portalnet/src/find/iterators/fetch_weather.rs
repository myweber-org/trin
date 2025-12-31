use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct WeatherData {
    temperature: f64,
    humidity: f64,
    description: String,
    timestamp: SystemTime,
}

struct WeatherCache {
    data: HashMap<String, WeatherData>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&self, location: &str) -> Option<&WeatherData> {
        self.data.get(location).and_then(|weather| {
            if weather.timestamp.elapsed().unwrap_or(self.ttl) < self.ttl {
                Some(weather)
            } else {
                None
            }
        })
    }

    fn insert(&mut self, location: String, weather: WeatherData) {
        self.data.insert(location, weather);
    }

    fn clear_expired(&mut self) {
        let now = SystemTime::now();
        self.data.retain(|_, weather| {
            now.duration_since(weather.timestamp)
                .map(|duration| duration < self.ttl)
                .unwrap_or(false)
        });
    }
}

async fn fetch_weather_from_api(location: &str) -> Result<WeatherData, Box<dyn std::error::Error>> {
    let url = format!("https://api.weather.example/{}", location);
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API returned status: {}", response.status()).into());
    }

    let weather: WeatherData = response.json().await?;
    Ok(weather)
}

pub struct WeatherFetcher {
    cache: WeatherCache,
}

impl WeatherFetcher {
    pub fn new(cache_ttl_seconds: u64) -> Self {
        WeatherFetcher {
            cache: WeatherCache::new(cache_ttl_seconds),
        }
    }

    pub async fn get_weather(&mut self, location: &str) -> Result<WeatherData, Box<dyn std::error::Error>> {
        self.cache.clear_expired();

        if let Some(cached) = self.cache.get(location) {
            return Ok(WeatherData {
                temperature: cached.temperature,
                humidity: cached.humidity,
                description: cached.description.clone(),
                timestamp: cached.timestamp,
            });
        }

        let weather = fetch_weather_from_api(location).await?;
        let fresh_weather = WeatherData {
            temperature: weather.temperature,
            humidity: weather.humidity,
            description: weather.description,
            timestamp: SystemTime::now(),
        };

        self.cache.insert(location.to_string(), fresh_weather.clone());
        Ok(fresh_weather)
    }

    pub fn cache_stats(&self) -> usize {
        self.cache.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_weather_fetching() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/london")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"temperature": 15.5, "humidity": 65.0, "description": "cloudy"}"#)
            .create_async()
            .await;

        let mut fetcher = WeatherFetcher::new(300);
        let result = fetcher.get_weather("london").await;
        
        mock.assert();
        assert!(result.is_ok());
        let weather = result.unwrap();
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.humidity, 65.0);
        assert_eq!(weather.description, "cloudy");
    }
}