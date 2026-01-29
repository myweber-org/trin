use reqwest;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("API returned error: {0}")]
    ApiError(String),
    #[error("Invalid response format")]
    ParseError,
}

#[derive(Deserialize, Debug)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub condition: String,
}

pub struct WeatherClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl WeatherClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            api_key,
            base_url: "https://api.weatherservice.com/v1".to_string(),
        }
    }

    pub async fn fetch_weather(&self, city: &str) -> Result<WeatherData, WeatherError> {
        let url = format!("{}/current?city={}&key={}", self.base_url, city, self.api_key);
        
        for attempt in 1..=3 {
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let weather: WeatherData = response.json().await
                            .map_err(|_| WeatherError::ParseError)?;
                        return Ok(weather);
                    } else {
                        let error_text = response.text().await.unwrap_or_default();
                        if attempt == 3 {
                            return Err(WeatherError::ApiError(error_text));
                        }
                    }
                }
                Err(e) => {
                    if attempt == 3 {
                        return Err(WeatherError::RequestFailed(e));
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_secs(attempt * 2)).await;
        }
        
        Err(WeatherError::ApiError("Max retries exceeded".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_successful_fetch() {
        let _m = mock("GET", "/v1/current")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("city".into(), "London".into()),
                Matcher::UrlEncoded("key".into(), "test_key".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"temperature":15.5,"humidity":65.0,"condition":"Cloudy"}"#)
            .create();

        let client = WeatherClient {
            client: reqwest::Client::new(),
            api_key: "test_key".to_string(),
            base_url: mockito::server_url(),
        };

        let result = client.fetch_weather("London").await;
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.temperature, 15.5);
        assert_eq!(data.condition, "Cloudy");
    }
}