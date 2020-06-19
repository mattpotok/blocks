use std::env;
use std::process;

use weather;
use weather::I3Block;

fn main() {
    // Parse configuration file
    let args: Vec<String> = env::args().collect();
    let config = match weather::Config::new(&args) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };

    // Fetch external IP
    let ipv4 = match weather::IPv4::new(config.log_ip) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };

    // Fetch geolocation based on IP
    let location = match weather::GeoLocation::new(ipv4, config.log_geolocation) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };

    // Fetch weather report
    match weather::OpenWeatherReport::new(
        &location,
        config.open_weather_api_key,
        config.temperature_scale,
        config.log_weather_report,
    ) {
        Ok(report) => println!("{}", report.format_i3()),
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };
}
