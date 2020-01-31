use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::fmt;
use std::path;
use std::process;

use chrono;
use reqwest;
use serde::Deserialize;
use serde_yaml;

// TODO incorporate this
use std::net::{SocketAddr, ToSocketAddrs, TcpStream};
use std::time::Duration;

// TODO update version of request
// FIXME error message function names

// TODO remove the Debug
#[derive(Deserialize, Debug)]
pub struct Config {
    error_file_path: path::PathBuf,
    open_weather_api_key: String,
    temperature_units: char,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, Box<dyn Error>> {
        // Check number of args
        if args.len() != 2 {
            let e = "Error (Config::new) - not enough arguments";
            return Err(e.into());
        }

        // Open configuration file
        let file = match File::open(&args[1]) {
            Ok(v) => v,
            Err(e) => {
                let e = format!("Error (Config::new) - {}", e);
                return Err(e.into());
            }
        };

        // Parse configuration file
        let mut config: Config = match serde_yaml::from_reader(file) {
            Ok(v) => v,
            Err(e) => {
                let e = format!("Error (Config::new) - {}", e);
                return Err(e.into());
            }
        };

        // Verify `error_file`
        if config.error_file_path.is_dir() {
            let e = "Error (Config::new) - \
                specify file for parameter `error_file_path`.";
            return Err(e.into());
        }

        /* FIXME No need to explicitly create a file
        if !config.error_file_path.exists() {
            if let Err(e) = File::create(&config.error_file_path) {
                let e = format!("Error (Config::new) - {}", e);
                return Err(e.into());
            }
        }
        */

        // Verify `temperature_unit`
        config.temperature_units = config.temperature_units.to_ascii_uppercase();
        let unit = config.temperature_units;
        if !(unit == 'F' || unit == 'C'  || unit == 'K') {
            let e = format!("Error (Config::new) - \
                invalid temperature unit '{}', select from ['C', 'F', 'K'].", unit);
            return Err(e.into());
        }

        Ok(config)
    }
}

pub struct IPv4 (String);

impl IPv4 {
    pub fn new() -> Result<IPv4, Box<dyn Error>> {
        // Get IP from ipify.org
        let url = "https://api.ipify.org";
        let mut resp = match reqwest::get(url) {
            Ok(v) => v,
            Err(e) => {
                let e = format!("Error (get_ip) - {}", e);
                return Err(e.into());
            }
        };

        // Extract response body
        match resp.text() {
            Ok(v) => Ok(IPv4(v)),
            Err(e) => {
                let e = format!("Error (get_ip) - {}", e);
                Err(e.into())
            }
        }
    }
}

impl fmt::Display for IPv4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
       

// TODO remove the Debug
#[derive(Deserialize, Debug)]
pub struct GeoLocation {
    status: String,
    lat: f64,
    lon: f64,

    #[serde(default)]
    message: String,
}

// FIXME error strings
impl GeoLocation {
    pub fn new(ip: IPv4) -> Result<GeoLocation, Box<dyn Error>> {
        // Get Geoleocation
        let url = format!(
            "http://ip-api.com/json/{}?fields=status,message,lat,lon", ip);
        let mut resp = match reqwest::get(&url) {
            Ok(v) => v,
            Err(e) => {
                let e = format!("Error (get_geolocation) - {}", e);
                return Err(e.into());
            }
        };

        // FIXME rename err to e
        // Extract response body
        let location: GeoLocation = match resp.json() {
            Ok(v) => v,
            Err(err) => {
                let err = format!("Error (get_geolocation) - {}", err);
                return Err(err.into());
            }
        };

        // Check body status
        if location.status == "fail" {
            let err = format!("Error (get_geolocation) - ip-api: {}",
                              location.message);
            return Err(err.into());
        }

        Ok(location)
    }
}

#[derive(Deserialize, Debug)]
pub struct OpenWeatherWeather {
    main: String,
    description: String,
}

#[derive(Deserialize, Debug)]
pub struct OpenWeatherMain {
    temp: f32,

    #[serde(default)]
    scale: char,
}

#[derive(Deserialize, Debug)]
pub struct OpenWeatherReport {
    main: OpenWeatherMain,
    weather: Vec<OpenWeatherWeather>,
}


impl OpenWeatherReport {
    pub fn new(
        location: &GeoLocation, config: &Config
    ) -> Result<OpenWeatherReport, Box<dyn Error>> {
        // Get OpenWeather report
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}",
            location.lat, location.lon, config.open_weather_api_key);

        // FIXME temporary
        println!("Url - {}", url);

        let mut resp = match reqwest::get(&url) {
            Ok(v) => v,
            Err(e) => {
                let e = format!("Error (Weather::new) - {}", e);
                return Err(e.into());
            }
        };

        // Extract response body
        let mut report: OpenWeatherReport = match resp.json() {
            Ok(v) => v,
            Err(e) => {
                let e = format!("Error (last fn) - {}", e);
                return Err(e.into());
            }
        };

        // Convert temperature
        report.main.scale = config.temperature_units;
        match report.main.scale {
            'C' => report.main.temp -= 273.15,
            'F' => report.main.temp = 
                1.8 * (report.main.temp - 273.15) + 32.0,
            _ => (),
        }

        Ok(report)
    }

    pub fn fmt_i3(&self) -> String {
        let full_text = format!("WTR {:.1}°{}, {}", self.main.temp, self.main.scale,
                                self.weather[0].main.to_ascii_lowercase());
        let short_text = format!("WTR {:.1}°{}", self.main.temp, self.main.scale);
        let color = "#FFFFFF";
        
        format!("{}\n{}\n{}", full_text, short_text, color)
    }
}

pub fn generate_error() -> Result<(), Box<dyn Error>> {
    Err("An error".into())
}

// TODO make this available to all blocks (?)
// TODO generalize and remove return value
pub fn handle_error(error: Box<dyn Error>, config: &Config) {
        //-> Result<(), Box<dyn Error>> {
    // Try to open the file
    let file_desc = "WTR File error!\n\
                     WTR File Error!\n\
                     #FF0000";

    let mut file = match OpenOptions::new()
        .append(true)
        .create(true)
        .open(&config.error_file_path) {
        Ok(v) => v,
        Err(E) => {
            println!("{}", file_desc);
            return;
        }
    };

    /*
    let mut file = match File::open(&config.error_file_path) {
        Ok(v) => v,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => match File::create(&config.error_file_path) {
                Ok(v) => v,
                Err(_) => {
                   // return Err(file_desc.into()),
                   println!("{}", file_desc);
                   return;
                },
            },
            _ => {
                // return Err(file_desc.into()),
                println!("{}", file_desc);
                return;
            },
        },
    };
    */

    // Write error to file
    let dt = chrono::offset::Local::now().format("%a %b %e %T %Y").to_string();
    //let error_string = format!("{} [{}] {}", dt, "WTR", error);
    writeln!(file, "{} [{}] {}", dt, "WTR", error);

    /* TODO debug this
    if let Err(_) = file.write_all(error_string.as_bytes()) {
        println!("{}", file_desc);
        return;
    }
    */
    
    /*
    let mut file = File::open("/home/potok/debug.txt").unwrap();
    match file.write_all(b"Hello, world!") {
        Ok(_) => {},
        Err(e) => println!("Error - {}", e),
    };
    */


    /*
    let mut file = File::create("~/debug.txt").unwrap();
    file.write_all(b"Hello, world!").unwrap();
    */
    

    // Ok(())

    /*
    let mut file: File;

    let mut file = match File::open(&config.error_file_path) {
        Ok(v) => v,
        Err(e) => return
    };

    file.write
    */
}


pub fn check_online() -> bool {
    /*
    let addrs_iter = "www.google.com:80".to_socket_addrs();
    println!("Output - {:?}", addrs_iter);
    */

    /*
    let stream = TcpStream::connect("www.google.com:80");
    println!("Output - {:?}", stream);
    */

    let mut online = false;
    for letter in b'a'..=b'm' {
       let addr = format!("{}.root-servers.net:80", letter as char);
       if let Ok(_) = addr.to_socket_addrs() {
           println!("Address - {}", addr);
           online = true;
           break;
       }
    }

    online

    /*
    if let Some(_) = b'a'..=b'm'.iter().find_map(|l| {
           let addr = format!("{}.root-servers.net:80", l as char);
           addr.to_socket_addrs() }) {
        println!("Online!");
    } else {
        println!("Offline!");
    }
    */

    /*
    for letter in b'a'..=b'm' { 
        println!("URL - {}", format!("{}.root-servers.net:80", letter as char));
    }
    */

    /* TODO use the root servers rather than multiple companies (valid from ['a'...'m'])
    let addrs = [
        //"www.google.com:80", "www.baidu.com:80",
        "f.root-servers.net:80",
    ];

    // TODO use a map pattern here to return true on match (?)
    let mut online = false;
    for addr in &addrs {
        println!("Address - {}", addr);

        if let Ok(_) = addr.to_socket_addrs() {
            println!("Valid address - {}", addr);
            online = true;
            break;
        }
    }

    if online == false {
        println!("Offline!");
    } else {
        println!("Online!");
    }
    */

    /*
    //let addrs = ["8.8.8.8:80", "208.67.222.222:80", "1.1.1.1:80"]; 
    let addrs = [
        SocketAddr::from(([8, 8, 8, 8], 80)),
        SocketAddr::from(([208, 67, 222, 222], 80)),
        SocketAddr::from(([1, 1, 1, 1], 80)),
    ];


    for addr in &addrs {
        println!("Address - {}", addr);

        if let Err(e) = TcpStream::connect_timeout(addr, Duration::new(3, 0)) {
            println!("Unable to connect - {}!", e);
        } else {
            println!("Connected to {}!", addr);
        }
    }
    */

    /*
    if let Err(e) = TcpStream::connect(&addrs[..], Duration::new(3, 0)) {
        println!("Unable to connect - {}!", e);
        return;
    } else {
        println!("Connected!");
    }
    */
}

// TODO test by turning off the internet - may need to modify the error to know where it came from?
/*
pub fn get_ipv4() -> Result<String, Box<dyn Error>> {
    // Get IP from ipify.org
    let url = "https://api.ipify.org";
    let mut resp = match reqwest::get(url) {
        Ok(v) => v,
        Err(e) => {
            let e = format!("Error (get_ip) - {}", e);
            return Err(e.into());
        }
    };

    // Extract response body
    match resp.text() {
        Ok(ip) => Ok(ip),
        Err(e) => {
            let e = format!("Error (get_ip) - {}", e);
            return Err(e.into());
        }
    }
}
*/

/*
// FIXME make this be an impl for GeoLocation
pub fn get_geolocation(ip: String) -> Result<GeoLocation, Box<dyn Error>> {
    // Get Geoleocation
    let url = format!(
        "http://ip-api.com/json/{}?fields=status,message,lat,lon", ip);
    let mut resp = match reqwest::get(&url) {
        Ok(v) => v,
        Err(err) => {
            let err = format!("Error (get_geolocation) - {}", err);
            return Err(err.into());
        }
    };

    // Extract response body
    let geo_location: GeoLocation = match resp.json() {
        Ok(v) => v,
        Err(err) => {
            let err = format!("Error (get_geolocation) - {}", err);
            return Err(err.into());
        }
    };

    // Check body status
    if geo_location.status == "fail" {
        let err = format!("Error (get_geolocation) - ip-api: {}",
                          geo_location.message);
        return Err(err.into());
    }

    Ok(geo_location)
}
*/

