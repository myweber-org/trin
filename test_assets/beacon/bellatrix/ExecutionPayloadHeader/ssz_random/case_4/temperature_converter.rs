
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
}use std::io;

fn main() {
    println!("Temperature Converter");
    println!("Enter a temperature value followed by its unit (C or F), e.g., '25 C' or '77 F':");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.len() != 2 {
        println!("Invalid input format. Please enter a value and a unit (C or F).");
        return;
    }

    let value: f64 = match parts[0].parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid temperature value.");
            return;
        }
    };

    let unit = parts[1].to_uppercase();
    match unit.as_str() {
        "C" => {
            let fahrenheit = (value * 9.0 / 5.0) + 32.0;
            println!("{:.2}°C is {:.2}°F", value, fahrenheit);
        }
        "F" => {
            let celsius = (value - 32.0) * 5.0 / 9.0;
            println!("{:.2}°F is {:.2}°C", value, celsius);
        }
        _ => println!("Invalid unit. Please use 'C' for Celsius or 'F' for Fahrenheit."),
    }
}