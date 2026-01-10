use reqwest;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct WeatherData {
    temperature: f64,
    humidity: f64,
    description: String,
}

#[derive(Debug)]
pub enum WeatherError {
    NetworkError(reqwest::Error),
    ParseError(serde_json::Error),
    ApiError(String),
}

pub async fn fetch_weather_data(
    api_key: &str,
    city: &str,
    max_retries: u32,
) -> Result<WeatherData, WeatherError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(WeatherError::NetworkError)?;

    let url = format!(
        "https://api.weatherservice.com/v1/current?city={}&key={}",
        city, api_key
    );

    for attempt in 0..max_retries {
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let weather: WeatherData = response
                        .json()
                        .await
                        .map_err(WeatherError::ParseError)?;
                    return Ok(weather);
                } else {
                    if attempt == max_retries - 1 {
                        return Err(WeatherError::ApiError(
                            format!("API returned status: {}", response.status())
                        ));
                    }
                    tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                }
            }
            Err(e) => {
                if attempt == max_retries - 1 {
                    return Err(WeatherError::NetworkError(e));
                }
                tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
            }
        }
    }

    Err(WeatherError::ApiError("Max retries exceeded".to_string()))
}