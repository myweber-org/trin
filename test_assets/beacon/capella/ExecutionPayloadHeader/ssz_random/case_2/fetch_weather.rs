use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
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
    #[error("City not found")]
    CityNotFound,
    #[error("Cache serialization error")]
    CacheError,
}

struct WeatherCache {
    data: RwLock<HashMap<String, (WeatherResponse, SystemTime)>>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl: Duration) -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    fn get(&self, city: &str) -> Option<WeatherResponse> {
        let cache = self.data.read().unwrap();
        cache.get(city).and_then(|(response, timestamp)| {
            if timestamp.elapsed().unwrap() < self.ttl {
                Some(response.clone())
            } else {
                None
            }
        })
    }

    fn set(&self, city: String, response: WeatherResponse) {
        let mut cache = self.data.write().unwrap();
        cache.insert(city, (response, SystemTime::now()));
    }
}

pub struct WeatherFetcher {
    client: Client,
    api_key: String,
    cache: Arc<WeatherCache>,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            cache: Arc::new(WeatherCache::new(Duration::from_secs(300))),
        }
    }

    pub async fn fetch_weather(&self, city: &str) -> Result<WeatherResponse, WeatherError> {
        if let Some(cached) = self.cache.get(city) {
            return Ok(cached);
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let weather: WeatherResponse = response.json().await?;
            self.cache.set(city.to_string(), weather.clone());
            Ok(weather)
        } else {
            Err(WeatherError::CityNotFound)
        }
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.data.write().unwrap();
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();

        let fetcher = WeatherFetcher::new("test_key".to_string());
        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 15.5);
        assert_eq!(weather.main.humidity, 65);
    }

    #[tokio::test]
    async fn test_fetch_weather_city_not_found() {
        let _m = mock("GET", "/data/2.5/weather")
            .with_status(404)
            .create();

        let fetcher = WeatherFetcher::new("test_key".to_string());
        let result = fetcher.fetch_weather("UnknownCity").await;
        assert!(matches!(result, Err(WeatherError::CityNotFound)));
    }
}