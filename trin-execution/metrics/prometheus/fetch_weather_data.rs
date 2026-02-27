
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct WeatherData {
    main: Main,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Main {
    temp: f64,
    feels_like: f64,
    humidity: u8,
}

pub async fn get_weather(city: &str, api_key: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = reqwest::get(&url).await?;
    let weather: WeatherData = response.json().await?;

    Ok(weather)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_get_weather_success() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"name":"London","main":{"temp":15.5,"feels_like":14.2,"humidity":65}}"#)
            .create();

        let api_key = "test_key";
        let city = "London";
        let url = format!(
            "{}/data/2.5/weather?q={}&appid={}&units=metric",
            server.url(),
            city,
            api_key
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await.unwrap();
        let weather: WeatherData = response.json().await.unwrap();

        mock.assert();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 15.5);
        assert_eq!(weather.main.feels_like, 14.2);
        assert_eq!(weather.main.humidity, 65);
    }
}