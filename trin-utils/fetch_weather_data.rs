use reqwest;
use serde::Deserialize;
use std::error::Error;

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

pub async fn get_weather_data(api_key: &str, city: &str) -> Result<WeatherResponse, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather_data: WeatherResponse = response.json().await?;
    Ok(weather_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_get_weather_data_success() {
        let mock_response = r#"{
            "main": {"temp": 22.5, "humidity": 65},
            "name": "London"
        }"#;
        
        let _m = mock("GET", "/data/2.5/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_body(mock_response)
            .create();
        
        let result = get_weather_data("test_key", "London").await;
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert_eq!(data.name, "London");
        assert_eq!(data.main.temp, 22.5);
        assert_eq!(data.main.humidity, 65);
    }

    #[tokio::test]
    async fn test_get_weather_data_failure() {
        let _m = mock("GET", "/data/2.5/weather?q=InvalidCity&appid=test_key&units=metric")
            .with_status(404)
            .create();
        
        let result = get_weather_data("test_key", "InvalidCity").await;
        assert!(result.is_err());
    }
}