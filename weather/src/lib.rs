use std::fmt;
use std::net::ToSocketAddrs;

use log::{error, info};
use reqwest;
use serde::Deserialize;
use serde_yaml;
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};

// Constants
const DEFAULT_ERROR: &str = "WTR Error!\nWTR Error!\n#FF0000";

// Traits
pub trait I3Block {
    fn format_i3(&self) -> String;
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_bool_false")]
    pub log_ip: bool,

    #[serde(default = "Config::default_bool_false")]
    pub log_geolocation: bool,

    #[serde(default = "Config::default_bool_false")]
    pub log_weather_report: bool,

    #[serde(default = "Config::default_bool_true")]
    check_connection: bool,

    log_file_path: std::path::PathBuf,

    pub open_weather_api_key: String,

    #[serde(default = "Config::default_temperature_scale")]
    pub temperature_scale: char,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, String> {
        // Check number of arguments
        if args.len() != 2 {
            let e = "WTR Args error!\nWTR Args error!\n#FF000";
            return Err(e.into());
        }

        // Open configuration file
        let file = match std::fs::File::open(&args[1]) {
            Ok(v) => v,
            Err(_) => {
                let e = "WTR File error!\nWTR File error!\n#FF000";
                return Err(e.into());
            }
        };

        // Parse configuration file
        let mut config: Config = match serde_yaml::from_reader(file) {
            Ok(v) => v,
            Err(_) => {
                let e = "WTR Parse error!\nWTR Parse error\n#FF0000";
                return Err(e.into());
            }
        };

        // Create logger
        let file = match std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&config.log_file_path)
        {
            Ok(v) => v,
            Err(_) => {
                let e = "WTR File error!\nWTR File error!\n#FF0000";
                return Err(e.into());
            }
        };

        let logger = ConfigBuilder::new()
            .set_time_format_str("%a %b %e %T %Y")
            .set_time_to_local(true)
            .build();

        if let Err(_) = WriteLogger::init(LevelFilter::Info, logger, file) {
            let e = "WTR Logger error!\nWTR Logger error!\n#FF000";
            return Err(e.into());
        }

        // Check access to Internet
        if config.check_connection && !check_connection() {
            return Err("WTR 404\nWTR F04\n#FFFFFF".into());
        }

        // Verify temperature scale
        config.temperature_scale = config.temperature_scale.to_ascii_uppercase();
        let unit = config.temperature_scale;
        if !(unit == 'F' || unit == 'C' || unit == 'K') {
            error!(
                "weather::Config::new: invalid temperature scale '{}', select from \
                ['C', 'F', 'K']",
                unit
            );
            return Err(DEFAULT_ERROR.into());
        }

        Ok(config)
    }

    fn default_bool_false() -> bool {
        false
    }

    fn default_bool_true() -> bool {
        true
    }

    fn default_temperature_scale() -> char {
        'F'
    }
}

pub struct IPv4(String);

impl IPv4 {
    pub fn new(log: bool) -> Result<IPv4, String> {
        // Get IP from ipify.org
        let url = "https://api.ipify.org";
        let resp = match reqwest::blocking::get(url) {
            Ok(v) => v,
            Err(e) => {
                error!("weather::IPv4::new: {}", e);
                return Err(DEFAULT_ERROR.into());
            }
        };

        // Extract response body
        match resp.text() {
            Ok(v) => {
                if log {
                    info!("weather::IPv4::new: external IP is {}", v);
                }
                Ok(IPv4(v))
            }
            Err(e) => {
                error!("weather::IPv4::new: {}", e);
                Err(DEFAULT_ERROR.into())
            }
        }
    }
}

impl fmt::Display for IPv4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize)]
pub struct GeoLocation {
    status: String,
    lat: f64,
    lon: f64,

    #[serde(default)]
    message: String,
}

impl GeoLocation {
    pub fn new(ip: IPv4, log: bool) -> Result<GeoLocation, String> {
        // Get Geoleocation
        let url = format!(
            "http://ip-api.com/json/{}?fields=status,message,lat,lon",
            ip
        );
        let resp = match reqwest::blocking::get(&url) {
            Ok(v) => v,
            Err(e) => {
                error!("weather::GeoLocation::new: {}", e);
                return Err(DEFAULT_ERROR.into());
            }
        };

        // Extract response body
        let location: GeoLocation = match resp.json() {
            Ok(v) => v,
            Err(e) => {
                error!("weather::GeoLocation::new: {}", e);
                return Err(DEFAULT_ERROR.into());
            }
        };

        // Check body status
        if location.status == "fail" {
            error!("weather::GeoLocation::new: {}", location.message);
            return Err(DEFAULT_ERROR.into());
        }

        if log {
            info!("weather::GeoLocation::new: geolocation is {}", location);
        }

        Ok(location)
    }
}

impl fmt::Display for GeoLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "lat: {:.3}, lon: {:.3}", self.lat, self.lon)
    }
}

#[derive(Deserialize)]
pub struct OpenWeatherWeather {
    main: String,
}

#[derive(Deserialize)]
pub struct OpenWeatherMain {
    temp: f32,

    #[serde(default)]
    scale: char,
}

#[derive(Deserialize)]
pub struct OpenWeatherReport {
    main: OpenWeatherMain,
    weather: Vec<OpenWeatherWeather>,
}

impl OpenWeatherReport {
    pub fn new(
        location: &GeoLocation,
        api_key: String,
        temperature_scale: char,
        log: bool,
    ) -> Result<OpenWeatherReport, String> {
        // Get OpenWeather report
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}",
            location.lat, location.lon, api_key
        );
        let resp = match reqwest::blocking::get(&url) {
            Ok(v) => v,
            Err(e) => {
                error!("weather::OpenWeatherReport::new: {}", e);
                return Err(DEFAULT_ERROR.into());
            }
        };

        // Extract response body
        let mut report: OpenWeatherReport = match resp.json() {
            Ok(v) => v,
            Err(e) => {
                error!("weather::OpenWeatherReport::new: {}", e);
                return Err(DEFAULT_ERROR.into());
            }
        };

        // Convert temperature
        report.main.scale = temperature_scale;
        match report.main.scale {
            'C' => report.main.temp -= 273.15,
            'F' => report.main.temp = 1.8 * (report.main.temp - 273.15) + 32.0,
            _ => (),
        }

        // Log information
        if log {
            info!(
                "weather::OpenWeatherReport::new: \
                  current weather is {:.1}°{}, {}",
                report.main.temp,
                report.main.scale,
                report.weather[0].main.to_ascii_lowercase()
            );
        }

        Ok(report)
    }
}

impl I3Block for OpenWeatherReport {
    fn format_i3(&self) -> String {
        let full_text = format!(
            "WTR {:.1}°{}, {}",
            self.main.temp,
            self.main.scale,
            self.weather[0].main.to_ascii_lowercase()
        );
        let short_text = format!("WTR {:.1}°{}", self.main.temp, self.main.scale);
        let color = "#FFFFFF";

        format!("{}\n{}\n{}", full_text, short_text, color)
    }
}

fn check_connection() -> bool {
    // Check connection to 'root-servers'
    for letter in b'a'..=b'm' {
        let addr = format!("{}.root-servers.net:80", letter as char);
        if let Ok(_) = addr.to_socket_addrs() {
            return true;
        }
    }

    false
}
