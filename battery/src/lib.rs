use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

// TODO change the name of this
pub struct Config {
    log_file_path: String,
}

// TODO add the units to each
#[derive(Debug, Default)]
pub struct Battery {
    charge_full: Option<u64>,
    charge_full_design: Option<u64>,
    charge_now: Option<u64>,
    current_now: Option<u64>,
    status: Option<String>,

    capacity_design: u64,
    capacity_full: u64,
    capacity_now: u64,
    charge_status: String,
    present_rate: u64,
    time_remaining: u64,
}

impl Battery {
    pub fn new(path: PathBuf) -> Option<Battery> {
        let mut battery = Battery::default();

        // TODO log!
        let entries = match path.read_dir() {
            Ok(v) => v,
            // Err(_) => return Err(String::from("Error!")),
            Err(_) => return None,
        };

        // Parse files
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(entry_name) = entry.file_name().to_str() {
                    let entry_path = entry.path();
                    match entry_name {
                        "charge_full" =>
                            battery.charge_full = parse_file::<u64>(entry_path)
                                .map(|v| v / 1000),
                        "charge_full_design" => 
                            battery.charge_full_design = parse_file::<u64>(entry_path)
                                .map(|v| v / 1000),
                        "charge_now" =>
                            battery.charge_now = parse_file::<u64>(entry_path)
                                .map(|v| v / 1000),
                        "current_now" =>
                            battery.current_now = parse_file::<u64>(entry_path)
                                .map(|v| v / 1000),
                        "status" =>
                            battery.status = parse_file(entry_path),
                        _ => (),
                    }
                }
            }
        }

        // TODO log at each level here

        // Determine battery's current capacity
        if battery.charge_full.is_some() {
            battery.capacity_full = battery.charge_full.unwrap();
        } else {
            return None;
        }

        // Determine battery's design capacity
        if battery.charge_full_design.is_some() {
            battery.capacity_design = battery.charge_full_design.unwrap();
        } else {
            return None;
        }

        // Determine battery's remaining capacity
        if battery.charge_now.is_some() {
            battery.capacity_now = battery.charge_now.unwrap();
        } else {
            return None;
        }

        // Determine battery's (dis)charging rate
        if battery.current_now.is_some() {
            battery.present_rate = battery.current_now.unwrap();
        } else {
            return None;
        }

        // Determine battery's state
        if let Some(status) = battery.status.clone() {
            match status.to_ascii_lowercase().as_str() {
                "charging" => {
                    battery.charge_status = String::from("charging");
                    battery.time_remaining = 3600 *
                        (battery.capacity_full - battery.capacity_now) /
                        battery.present_rate;
                },
                "discharging" => {
                    battery.charge_status = String::from("discharging");
                    battery.time_remaining = 3600 *
                        battery.capacity_now / battery.present_rate;
                },
                "full" => battery.charge_status = String::from("full"),
                _ => battery.charge_status = String::from("unknown"),
            };
        } else {
            battery.charge_status = String::from("unknown");
        }

        Some(battery)
    }
}

// TODO create a struct `Batteries` to handle multiple batteries
#[derive(Debug, Default)]
pub struct Batteries {
    cells: Vec<Battery>,

    capacity_percent: f64,
    charge_percent: f64,
    charge_status: String,
    time_remaining: u64
}

// TODO add a path here! This will allow testing!
impl Batteries {
    pub fn new() -> Result<Batteries, String> {
        let mut batteries = Batteries::default();

        let path = Path::new("/sys/class/power_supply");
        let entries = match path.read_dir() {
            Ok(v) => v,
            Err(_) => return Err(String::from("Error!")),
        };

        // Gather cells
        let mut lcm_capacity_design: u64 = 1;
        let mut lcm_capacity_full: u64 = 1;
        let mut charge_status_mask: u8 = 0;

        // TODO reduce the nesting here!
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(entry_name) = entry.file_name().to_str() {
                    if entry_name.starts_with("BAT") {
                        // TODO return on Option rather than error (?)
                        // And log the error instead (?)
                        if let Some(battery) = Battery::new(entry.path()) {
                            
                            lcm_capacity_design =
                                (lcm_capacity_design * battery.capacity_design)
                                / gcd(lcm_capacity_design, battery.capacity_design);

                            lcm_capacity_full =
                                (lcm_capacity_full * battery.capacity_full)
                                / gcd(lcm_capacity_full, battery.capacity_full);

                            match battery.charge_status.as_str() {
                                "charging" => charge_status_mask |= 1,
                                "full" => charge_status_mask |= 2,
                                "discharging" => charge_status_mask |= 4,
                                _ => charge_status_mask |= 8,
                            }
                            
                            batteries.cells.push(battery);
                        }
                    }
                }
            }
        }

        // TODO set the batteries.charge_status here instead
        // Valid cases:
        //  charging and full only
        //  full only
        //  discharging

        if charge_status_mask == 2 {
            // Batteries are all full
            batteries.charge_status = String::from("full");
        } else if charge_status_mask == 3 {
            // Batteries are all charging or full
            batteries.charge_status = String::from("charging");
        } else if charge_status_mask == 4 {
            // Batteries are all discharging
            batteries.charge_status = String::from("discharging");
        }

        // Gather information
        let mut capacity_full: u64 = 0;
        let mut capacity_now: u64 = 0;
        for cell in &batteries.cells {
            let mult_dc = lcm_capacity_design / cell.capacity_design;
            capacity_full += mult_dc * cell.capacity_full;

            let mult_cc = lcm_capacity_full / cell.capacity_full;
            capacity_now += mult_cc * cell.capacity_now;

            if batteries.charge_status == "charging" ||
                batteries.charge_status == "discharging" {
                batteries.time_remaining += cell.time_remaining;
            }
        }

        batteries.capacity_percent = 100.0 * (capacity_full as f64) / 
            ((lcm_capacity_design * (batteries.cells.len() as u64)) as f64);

        batteries.charge_percent = 100.0 * (capacity_now as f64) / 
            ((lcm_capacity_full * (batteries.cells.len() as u64)) as f64);

        println!("Capacity - {}", batteries.capacity_percent);
        println!("Charge - {}", batteries.charge_percent);
        println!("Status - {}", batteries.charge_status);
        println!("Time - {}", batteries.time_remaining);

        Ok(batteries)
    }
}



pub fn read_file() {
    let path = "/sys/class/power_supply/BAT0/charge_now";

    let content = fs::read_to_string(path);
    println!("Content - {:?}", content);
}

pub fn read_file_generic<T>(path: &str) -> Result<T, Box<dyn Error>>
    where T: std::str::FromStr + std::fmt::Debug {
    let contents = fs::read_to_string(path).unwrap(); // TODO handle this better
    let trimmed = contents.trim();

    let value = match trimmed.parse::<T>() {
        Ok(v) => v,
        Err(_) => {
            let e = "Error unable to convert to requested type!";
            return Err(e.into());
        }
    };

    println!("Contents - {:?}", contents);
    println!("Value - {:?}", value);

    return Ok(value);
}

// TODO remove the debug option
fn read_file_v2<T>(path: &str) -> Option<T>
        where T: std::str::FromStr + std::fmt::Debug {
    let contents = match fs::read_to_string(path) {
        Ok(v) => v,
        Err(_) => return None,
    };

    let value = match contents.trim().parse::<T>() {
        Ok(v) => v,
        Err(_) => return None,
    };

    Some(value)
}

fn parse_file<T>(path: PathBuf) -> Option<T>
        where T: std::str::FromStr + std::fmt::Debug {
    let contents = match fs::read_to_string(path) {
        Ok(v) => v,
        Err(_) => return None,
    };

    match contents.trim().parse::<T>() {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

fn gcd(mut u: u64, mut v: u64) -> u64 {
    if u == 0 {
        return v;
    } else if v == 0 {
        return u;
    }

    let shift = (u | v).trailing_zeros();
    u >>= shift;
    v >>= shift;
    u >>= u.trailing_zeros();

    loop {
        v >>= v.trailing_zeros();

        if u > v {
            let t = u;
            u = v;
            v = t;
        }

        v -= u;
        if v == 0 {
            break;
        }
    }

    u << shift
}
