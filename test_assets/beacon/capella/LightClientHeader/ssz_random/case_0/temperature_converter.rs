
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

pub fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celsius_to_fahrenheit() {
        assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
        assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
        assert_eq!(celsius_to_fahrenheit(-40.0), -40.0);
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        assert_eq!(fahrenheit_to_celsius(32.0), 0.0);
        assert_eq!(fahrenheit_to_celsius(212.0), 100.0);
        assert_eq!(fahrenheit_to_celsius(-40.0), -40.0);
    }
}
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
    let celsius_temp = 25.0;
    println!("{:.1}°C = {:.1}°F", celsius_temp, celsius_to_fahrenheit(celsius_temp));
    println!("{:.1}°C = {:.1}K", celsius_temp, celsius_to_kelvin(celsius_temp));
    
    let fahrenheit_temp = 77.0;
    println!("{:.1}°F = {:.1}°C", fahrenheit_temp, fahrenheit_to_celsius(fahrenheit_temp));
    println!("{:.1}°F = {:.1}K", fahrenheit_temp, fahrenheit_to_kelvin(fahrenheit_temp));
    
    let kelvin_temp = 298.15;
    println!("{:.1}K = {:.1}°C", kelvin_temp, kelvin_to_celsius(kelvin_temp));
    println!("{:.1}K = {:.1}°F", kelvin_temp, kelvin_to_fahrenheit(kelvin_temp));
}