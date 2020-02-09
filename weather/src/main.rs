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
    // Temporary
    //weather::check_online(); 
    
    /*
    let file = match OpenOptions::new()
        .append(true)
        .create(true)
        .open("/home/potok/log.txt") {
        Ok(v) => v,
        Err(_) => {
            println!("Unable to open file!");
            return;
        }
    };

    // TODO modify the configuration here
    let config = ConfigBuilder::new()
        .set_time_format_str("%a %b %e %T %Y")
        .set_time_to_local(true)
        .build();
     
    WriteLogger::init(LevelFilter::Debug, config, file);
    info!("Test an 'info'");
    debug!("Test an 'debug'");
    error!("Test an 'error'");

    return;
    */


    // Parse configuration file
    let args: Vec<String> = env::args().collect();
    let config = match weather::Config::new(&args) {
        Ok(v) => v,
        Err(s) => {
            println!("{}", s);
            process::exit(1);
        }
    };

    // TODO add option to check if have an internet connection

    // Initialize the logger
    // TODO handle any errors with logger initialization
    /*
    if let Err(_) = weather::initialize_logger(&config) {
        println!("WTR Logger Error!\nWTR Logger Error!\n#FF0000");
        process::exit(1);
    }
    */

    // Fetch external IP
    let ipv4 = match weather::IPv4::new() {
        Ok(v) => v,
        Err(e) => {
            // FIXME remove this!
            error!("({}) {}", "WTR", e);
            //weather::handle_error(e, &config);
            println!("WTR IPv4 Error!\nWTR IPv4 Error!\n#FF0000");
            process::exit(1);
        }
    };

    // Make this an optional info
    println!("IP address - {}", ipv4);

    // Fetch geolocation based on IP
    let location = match weather::GeoLocation::new(ipv4) {
        Ok(v) => v,
        Err(e) => {
            weather::handle_error(e, &config);
            process::exit(1);
        }
    };

    println!("Geolocation - {:?}", location);

    let report = match weather::OpenWeatherReport::new(&location, &config) {
        Ok(v) => v,
        Err(e) => {
            weather::handle_error(e, &config);
            process::exit(1);
        }
    };


    println!("{}", report.fmt_i3());


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
