extern crate regex;

use std::process::Command;
use regex::Regex;

#[derive(Default)]
struct Battery {
    charge: u8,
    design_capacity: u16,
    full_capacity: u16,
    state: String,
    time: String,
}

fn main() {
    // Variables
    let mut battery: Battery = Default::default();

    // Call 'acpi -bi'
    let result = Command::new("acpi")
                         .arg("-bi")
                         .output();
    if result.is_err() {
        println!("BAT 'acpi' error\nBAT Error\n#FF0000");
        return;
    }

    // Convert output to a string
    let buffer = result.unwrap().stdout;
    let result = String::from_utf8(buffer).unwrap();
        
    // Define regexes
    let re_num = Regex::new(r"\d+").unwrap();
    let re_time = Regex::new(r"(\d{2}:?)+").unwrap();

    // Parse results
    let lines = result.lines();
    for line in lines {
        
        let info = match line.split(": ").nth(1) {
            Some(v) => v,
            None => {
                println!("BAT Parse error\nBAT Error\n#FF0000");
                return;
            }
        };

        // Extract information
        let mut parts = info.split(", "); 
        let mut part = parts.next().unwrap();
        if part.starts_with("design") == true {
            // Extract 'design capacity'
            let mut mat = re_num.find(part).unwrap();
            battery.design_capacity = part[mat.start()..mat.end()].parse().unwrap();

            // Extract 'full capacity'
            part = parts.next().unwrap();
            mat = re_num.find(part).unwrap();
            battery.full_capacity = mat.as_str().parse().unwrap();
        } else {
            // Extract 'state'
            battery.state = String::from(part);

            // Extract 'charge'
            part = parts.next().unwrap();
            let mat = re_num.find(part).unwrap();
            battery.charge = mat.as_str().parse().unwrap();

            // Extract 'remaining time'
            let part = parts.next();
            if part.is_some() {
                let part = part.unwrap();
                match re_time.find(part) {
                    Some(mat) => {
                        let mat = mat.as_str();
                        let idx = mat.len() - 3;
                        battery.time = mat.chars().take(idx).collect();
                    },
                    None => {
                        battery.time = String::from("00:00");
                    },
                }
            }
        }
    }

    // Gather info
    let state = match &battery.state[..] {
        "Charging" => String::from("CHR"),
        "Discharging" => String::from("DIS"),
        "Full" => String::from("FUL"),
        "Unknown" => String::from ("UKN"),
        _ => String::from("ERR"),
    };

    let capacity = (battery.full_capacity as f64) / (battery.design_capacity as f64) * 100.0;

    // Create output
    let mut full_text = format!("BAT {}% ({:.0}%)  {}", battery.charge, capacity, state);
    let mut short_text = format!("Bat {}%", battery.charge);
    if state == "CHR" || state == "DIS" {
        full_text = format!("{}  {}", full_text, battery.time);
        short_text = format!("{} {}", short_text, battery.time);
    }

    // TODO do a filter or find here (?)
    // Color, computed via HSL
    let mut color = "#FFFFFF";
    if state == "DIS" {
        if battery.charge >= 90 {
            color = "#00FF00";
        } else if battery.charge >= 80 {
            color = "#37FF00";
        } else if battery.charge >= 70 {
            color = "#75FF00"
        } else if battery.charge >= 60 {
            color = "AAFF00";
        } else if battery.charge >= 50 {
            color = "#E1FF00";
        } else if battery.charge >= 40 {
            color = "#FFE100";
        } else if battery.charge >= 30 {
            color = "#FFAA00";
        } else if battery.charge >= 20 {
            color = "#FF7300";
        } else if battery.charge >= 10 {
            color = "#FF3700";
        } else {
            color = "#FF000";
        }
    }
    
    // Print output
    println!("{}\n{}\n{}", full_text, short_text, color);
}
