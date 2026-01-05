
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

fn fahrenheit_to_kelvin(fahrenheit: f64) -> f64 {
    celsius_to_kelvin(fahrenheit_to_celsius(fahrenheit))
}

fn kelvin_to_celsius(kelvin: f64) -> f64 {
    kelvin - 273.15
}

fn kelvin_to_fahrenheit(kelvin: f64) -> f64 {
    celsius_to_fahrenheit(kelvin_to_celsius(kelvin))
}

fn main() {
    let test_celsius = 25.0;
    println!("{}°C = {}°F", test_celsius, celsius_to_fahrenheit(test_celsius));
    println!("{}°C = {}K", test_celsius, celsius_to_kelvin(test_celsius));
    
    let test_fahrenheit = 77.0;
    println!("{}°F = {}°C", test_fahrenheit, fahrenheit_to_celsius(test_fahrenheit));
    println!("{}°F = {}K", test_fahrenheit, fahrenheit_to_kelvin(test_fahrenheit));
    
    let test_kelvin = 298.15;
    println!("{}K = {}°C", test_kelvin, kelvin_to_celsius(test_kelvin));
    println!("{}K = {}°F", test_kelvin, kelvin_to_fahrenheit(test_kelvin));
}