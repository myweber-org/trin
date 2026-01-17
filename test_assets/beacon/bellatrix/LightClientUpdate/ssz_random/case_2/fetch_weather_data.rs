use reqwest;
use serde::Deserialize;
use std::env;

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

pub async fn get_current_weather(city: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENWEATHER_API_KEY")?;
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = reqwest::get(&url).await?.json::<WeatherResponse>().await?;
    
    Ok(format!(
        "Current weather in {}: {:.1}°C, {}% humidity",
        response.name, response.main.temp, response.main.humidity
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_weather_parsing() {
        let _m = mock("GET", "/data/2.5/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();

        std::env::set_var("OPENWEATHER_API_KEY", "test_key");
        let result = get_current_weather("London").await.unwrap();
        assert_eq!(result, "Current weather in London: 15.5°C, 65% humidity");
    }
}