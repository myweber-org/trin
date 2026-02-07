use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::Deserialize;

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

struct WeatherCache {
    data: HashMap<String, (WeatherResponse, SystemTime)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&self, city: &str) -> Option<&WeatherResponse> {
        self.data.get(city).and_then(|(response, timestamp)| {
            if timestamp.elapsed().unwrap_or(self.ttl) < self.ttl {
                Some(response)
            } else {
                None
            }
        })
    }

    fn insert(&mut self, city: String, response: WeatherResponse) {
        self.data.insert(city, (response, SystemTime::now()));
    }
}

async fn fetch_weather(api_key: &str, city: &str) -> Result<WeatherResponse, reqwest::Error> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    response.json::<WeatherResponse>().await
}

pub async fn get_weather(
    cache: &mut WeatherCache,
    api_key: &str,
    city: &str,
) -> Result<WeatherResponse, String> {
    if let Some(cached) = cache.get(city) {
        return Ok(cached.clone());
    }

    match fetch_weather(api_key, city).await {
        Ok(weather) => {
            cache.insert(city.to_string(), weather.clone());
            Ok(weather)
        }
        Err(e) => Err(format!("Failed to fetch weather: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_weather_fetch() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create_async()
            .await;

        let api_key = "test_key";
        let city = "London";
        
        let mut cache = WeatherCache::new(300);
        let result = get_weather(&mut cache, api_key, city).await;
        
        mock.assert();
        assert!(result.is_ok());
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 15.5);
        assert_eq!(weather.main.humidity, 65);
    }
}