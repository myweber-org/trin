
use reqwest;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
struct WeatherData {
    main: Main,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Main {
    temp: f64,
    humidity: u8,
}

pub async fn get_weather(city: &str) -> Result<WeatherData, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENWEATHER_API_KEY")
        .expect("OPENWEATHER_API_KEY environment variable not set");
    
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    let weather: WeatherData = response.json().await?;
    
    Ok(weather)
}

pub fn display_weather(weather: &WeatherData) {
    println!("Weather in {}:", weather.name);
    println!("Temperature: {:.1}°C", weather.main.temp);
    println!("Humidity: {}%", weather.main.humidity);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_get_weather() {
        let _m = mock("GET", "/data/2.5/weather")
            .match_query(Matcher::Regex(r"q=London.*".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();

        std::env::set_var("OPENWEATHER_API_KEY", "test_key");
        
        let result = get_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 15.5);
        assert_eq!(weather.main.humidity, 65);
    }
}