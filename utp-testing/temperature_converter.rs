fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    (c * 9.0 / 5.0) + 32.0
}

fn main() {
    let f_temp = 68.0;
    let c_temp = fahrenheit_to_celsius(f_temp);
    println!("{:.1}°F is {:.1}°C", f_temp, c_temp);

    let c_temp2 = 20.0;
    let f_temp2 = celsius_to_fahrenheit(c_temp2);
    println!("{:.1}°C is {:.1}°F", c_temp2, f_temp2);
}