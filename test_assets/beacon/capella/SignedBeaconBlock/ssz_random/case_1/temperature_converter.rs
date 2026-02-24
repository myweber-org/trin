
use std::io;

enum TemperatureUnit {
    Celsius,
    Fahrenheit,
    Kelvin,
}

struct Temperature {
    value: f64,
    unit: TemperatureUnit,
}

impl Temperature {
    fn to_celsius(&self) -> f64 {
        match self.unit {
            TemperatureUnit::Celsius => self.value,
            TemperatureUnit::Fahrenheit => (self.value - 32.0) * 5.0 / 9.0,
            TemperatureUnit::Kelvin => self.value - 273.15,
        }
    }

    fn to_fahrenheit(&self) -> f64 {
        match self.unit {
            TemperatureUnit::Celsius => (self.value * 9.0 / 5.0) + 32.0,
            TemperatureUnit::Fahrenheit => self.value,
            TemperatureUnit::Kelvin => (self.value - 273.15) * 9.0 / 5.0 + 32.0,
        }
    }

    fn to_kelvin(&self) -> f64 {
        match self.unit {
            TemperatureUnit::Celsius => self.value + 273.15,
            TemperatureUnit::Fahrenheit => (self.value - 32.0) * 5.0 / 9.0 + 273.15,
            TemperatureUnit::Kelvin => self.value,
        }
    }

    fn convert(&self, target_unit: TemperatureUnit) -> Temperature {
        let converted_value = match target_unit {
            TemperatureUnit::Celsius => self.to_celsius(),
            TemperatureUnit::Fahrenheit => self.to_fahrenheit(),
            TemperatureUnit::Kelvin => self.to_kelvin(),
        };
        
        Temperature {
            value: converted_value,
            unit: target_unit,
        }
    }
}

fn parse_temperature_input(input: &str) -> Option<Temperature> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let value: f64 = match parts[0].parse() {
        Ok(v) => v,
        Err(_) => return None,
    };

    let unit = match parts[1].to_lowercase().as_str() {
        "c" | "celsius" => TemperatureUnit::Celsius,
        "f" | "fahrenheit" => TemperatureUnit::Fahrenheit,
        "k" | "kelvin" => TemperatureUnit::Kelvin,
        _ => return None,
    };

    Some(Temperature { value, unit })
}

fn display_temperature(temp: &Temperature) -> String {
    let unit_str = match temp.unit {
        TemperatureUnit::Celsius => "°C",
        TemperatureUnit::Fahrenheit => "°F",
        TemperatureUnit::Kelvin => "K",
    };
    format!("{:.2} {}", temp.value, unit_str)
}

fn main() {
    println!("Temperature Converter");
    println!("Enter temperature and unit (e.g., '25 C', '77 F', '300 K')");
    println!("Type 'quit' to exit");

    loop {
        println!("\nEnter temperature (value unit):");
        
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let input = input.trim();
        if input.eq_ignore_ascii_case("quit") {
            break;
        }

        let source_temp = match parse_temperature_input(input) {
            Some(temp) => temp,
            None => {
                println!("Invalid input. Please use format: 'value unit' (e.g., '25 C')");
                continue;
            }
        };

        println!("Original: {}", display_temperature(&source_temp));
        println!("Converted to:");
        println!("  Celsius: {}", display_temperature(&source_temp.convert(TemperatureUnit::Celsius)));
        println!("  Fahrenheit: {}", display_temperature(&source_temp.convert(TemperatureUnit::Fahrenheit)));
        println!("  Kelvin: {}", display_temperature(&source_temp.convert(TemperatureUnit::Kelvin)));
    }

    println!("Goodbye!");
}fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn main() {
    let celsius_temp = 25.0;
    let fahrenheit_temp = celsius_to_fahrenheit(celsius_temp);
    println!("{}°C is equal to {}°F", celsius_temp, fahrenheit_temp);
}