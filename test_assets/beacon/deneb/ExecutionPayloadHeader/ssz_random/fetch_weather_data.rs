use reqwest;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct WeatherData {
    temperature: f64,
    humidity: f64,
    description: String,
}

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("Invalid response format")]
    InvalidFormat,
}

pub struct WeatherFetcher {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            api_key,
            base_url: "https://api.weather.example.com".to_string(),
        }
    }

    pub async fn fetch_weather(&self, city: &str) -> Result<WeatherData, WeatherError> {
        let url = format!("{}/current?city={}&key={}", self.base_url, city, self.api_key);
        
        for attempt in 1..=3 {
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let weather: WeatherData = response.json().await
                            .map_err(|_| WeatherError::InvalidFormat)?;
                        return Ok(weather);
                    } else {
                        let status = response.status();
                        if attempt < 3 && status.is_server_error() {
                            tokio::time::sleep(Duration::from_secs(attempt as u64)).await;
                            continue;
                        }
                        return Err(WeatherError::Api(format!("HTTP {}: {}", status, response.text().await.unwrap_or_default())));
                    }
                }
                Err(e) => {
                    if attempt < 3 {
                        tokio::time::sleep(Duration::from_secs(attempt as u64)).await;
                        continue;
                    }
                    return Err(WeatherError::Network(e));
                }
            }
        }
        
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/current?city=London&key=test_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"temperature": 15.5, "humidity": 65.0, "description": "Cloudy"}"#)
            .create();
        
        let fetcher = WeatherFetcher {
            client: reqwest::Client::new(),
            api_key: "test_key".to_string(),
            base_url: server_url(),
        };
        
        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        let weather = result.unwrap();
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.humidity, 65.0);
        assert_eq!(weather.description, "Cloudy");
    }
}