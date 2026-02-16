use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API response error: {0}")]
    Api(String),
    #[error("Cache error")]
    Cache,
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

pub struct WeatherCache {
    data: HashMap<String, (WeatherResponse, Instant)>,
    ttl: Duration,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, city: &str) -> Option<&WeatherResponse> {
        self.data.get(city)
            .filter(|(_, timestamp)| timestamp.elapsed() < self.ttl)
            .map(|(data, _)| data)
    }

    pub fn insert(&mut self, city: String, response: WeatherResponse) {
        self.data.insert(city, (response, Instant::now()));
    }
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: WeatherCache,
    client: reqwest::Client,
}

impl WeatherFetcher {
    pub fn new(api_key: String, cache_ttl: u64) -> Self {
        Self {
            api_key,
            base_url: "https://api.openweathermap.org/data/2.5/weather".to_string(),
            cache: WeatherCache::new(cache_ttl),
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_weather(&mut self, city: &str) -> Result<WeatherResponse, WeatherError> {
        if let Some(cached) = self.cache.get(city) {
            return Ok(cached.clone());
        }

        let url = format!(
            "{}?q={}&appid={}&units=metric",
            self.base_url, city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(WeatherError::Api(format!("HTTP {}: {}", response.status(), error_text)));
        }

        let weather_data: WeatherResponse = response.json().await?;
        self.cache.insert(city.to_string(), weather_data.clone());
        
        Ok(weather_data)
    }

    pub fn display_weather(weather: &WeatherResponse) -> String {
        let temp = weather.main.temp;
        let humidity = weather.main.humidity;
        let description = &weather.weather[0].description;
        let city = &weather.name;

        format!(
            "Weather in {}: {:.1}°C, {}% humidity, {}",
            city, temp, humidity, description
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/data/2.5/weather")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("q".into(), "London".into()),
                Matcher::UrlEncoded("appid".into(), "test_key".into()),
                Matcher::UrlEncoded("units".into(), "metric".into()),
            ]))
            .with_status(200)
            .with_body(r#"{
                "name": "London",
                "main": {"temp": 15.5, "humidity": 65, "pressure": 1013},
                "weather": [{"description": "clear sky", "icon": "01d"}]
            }"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string(), 300);
        fetcher.base_url = mockito::server_url();
        
        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 15.5);
        assert_eq!(weather.main.humidity, 65);
    }

    #[test]
    fn test_cache_behavior() {
        let mut cache = WeatherCache::new(60);
        let weather_data = WeatherResponse {
            name: "TestCity".to_string(),
            main: MainData {
                temp: 20.0,
                humidity: 50,
                pressure: 1000,
            },
            weather: vec![WeatherInfo {
                description: "sunny".to_string(),
                icon: "01d".to_string(),
            }],
        };

        assert!(cache.get("TestCity").is_none());
        cache.insert("TestCity".to_string(), weather_data);
        assert!(cache.get("TestCity").is_some());
    }
}