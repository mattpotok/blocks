//use serde::Deserialize;
//use serde_json::{Result, Value};
use std::env;
use std::process;

use weather;

// Testing
use std::fs::{OpenOptions, File};
use log::*;
//use simplelog::{Config, LevelFilter, WriteLogger};
use simplelog::*;


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

    /*
    // Get weather
    let weather = match weather::get_weather(location) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error - {}", e);
            process::exit(1);
        }
    };
    */
}

// ----- OLDv2 -----
/*
    // Get external IPv4 address
    let ipv4 = match weather::get_ipv4() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error - {}", e);
            process::exit(1);
        }
    };

    println!("IP - {}", ipv4);

    // Get geolocation
    let location = match weather::get_geolocation(ipv4) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error - {}", e);
            process::exit(1);
        }
    };

    println!("GeoLocation - {:?}", location);


*/


// ----- OLD -----
/*
    println!("Hello, world!");
    
    let response = reqwest::get("https://api.ipify.org?format=json");
    println!("Response - {:?}", response);

    // TODO test by turning off the internet
    let mut body = match response {
        Ok(v) => v,
        Err(e) => {
            println!("Error 1 - {:?}", e);
            process::exit(1); 
        }

    };

    let text = body.text();
    let data = match text {
        Ok(v) => v,
        Err(e) => {
            println!("Error 2 - {:?}", e);
            process::exit(1);
        }
    };

    // Parse the ip
    /*
    let ipv4: IPv4 = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(e) => {
            println!("Error 3 - {:?}", e);
            process::exit(1);
        }
    };
    */

    let value: Value = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(e) => {
            println!("Error 3 - {:?}", e);
            process::exit(1);
        }
    };

    let ipv4 = match value["ip"].as_str() {
        Some(v) => v,
        None => {
            println!("Error 4 - Unable to extract IP from JSON");
            process::exit(1);
        }
    };


    println!("IPv4 - {:?}", ipv4);


    /*
    let body = response.text();
    if let Err(e) = body {
        println!("Another error occurred - {:?}", e);
    }
    */
*/
