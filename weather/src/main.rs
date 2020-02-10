use std::env;
use std::process;

use weather;

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
    let ipv4 = match weather::IPv4::new(config.log_info) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };

    // Fetch geolocation based on IP
    let location = match weather::GeoLocation::new(ipv4, config.log_info) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };

    // Fetch weather report
    match weather::OpenWeatherReport::new(&location, &config) {
        Ok(report) => println!("{}", report.fmt_i3()),
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };
}
