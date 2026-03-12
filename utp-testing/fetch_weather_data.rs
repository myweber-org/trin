use reqwest;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("API returned an error: {0}")]
    ApiError(String),
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
}

#[derive(Deserialize, Debug)]
pub struct WeatherResponse {
    pub main: MainData,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct MainData {
    pub temp: f64,
    pub humidity: u8,
    pub pressure: u16,
}

pub struct WeatherFetcher {
    client: reqwest::Client,
    api_key: String,
    max_retries: u8,
    retry_delay: Duration,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        }
    }

    pub async fn fetch_weather(&self, city: &str) -> Result<WeatherResponse, WeatherError> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        for attempt in 0..self.max_retries {
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return response.json().await.map_err(WeatherError::RequestFailed);
                    } else {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        if status.is_server_error() && attempt < self.max_retries - 1 {
                            tokio::time::sleep(self.retry_delay * (attempt as u32 + 1)).await;
                            continue;
                        }
                        return Err(WeatherError::ApiError(format!(
                            "Status: {}, Error: {}",
                            status, error_text
                        )));
                    }
                }
                Err(e) => {
                    if attempt < self.max_retries - 1 {
                        tokio::time::sleep(self.retry_delay * (attempt as u32 + 1)).await;
                        continue;
                    }
                    return Err(WeatherError::RequestFailed(e));
                }
            }
        }

        Err(WeatherError::MaxRetriesExceeded)
    }

    pub fn set_max_retries(&mut self, retries: u8) {
        self.max_retries = retries;
    }

    pub fn set_retry_delay(&mut self, delay: Duration) {
        self.retry_delay = delay;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_successful_fetch() {
        let _m = mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"main":{"temp":20.5,"humidity":65,"pressure":1013},"name":"London"}"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.set_max_retries(1);
        
        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 20.5);
        assert_eq!(weather.main.humidity, 65);
        assert_eq!(weather.main.pressure, 1013);
    }

    #[tokio::test]
    async fn test_api_error() {
        let _m = mock("GET", "/data/2.5/weather")
            .with_status(404)
            .with_body(r#"{"message":"city not found"}"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.set_max_retries(1);
        
        let result = fetcher.fetch_weather("InvalidCity").await;
        assert!(matches!(result, Err(WeatherError::ApiError(_))));
    }
}