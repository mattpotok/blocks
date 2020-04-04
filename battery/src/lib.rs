use std::error::Error;
use std::fs;
use std::path::PathBuf;

use approx::relative_eq;

// TODO change the name of this
pub struct Config {
    log_file_path: String,
}

// TODO add the units to each
#[derive(Debug, Default)]
pub struct Battery {
    charge_full: Option<u64>,         // last 'full' charge (µAh)
    charge_full_design: Option<u64>,  // 'full' design charge (µAh)
    charge_now: Option<u64>,          // current charge (µAh)
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
    pub fn new(path: &PathBuf) -> Option<Battery> {
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
                            battery.charge_full = parse_file::<u64>(&entry_path)
                                .map(|v| v / 1000),
                        "charge_full_design" => 
                            battery.charge_full_design = parse_file::<u64>(&entry_path)
                                .map(|v| v / 1000),
                        "charge_now" =>
                            battery.charge_now = parse_file::<u64>(&entry_path)
                                .map(|v| v / 1000),
                        "current_now" =>
                            battery.current_now = parse_file::<u64>(&entry_path)
                                .map(|v| v / 1000),
                        "status" =>
                            battery.status = parse_file(&entry_path),
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
                    battery.time_remaining = (3600 *
                        (battery.capacity_full - battery.capacity_now)) /
                        battery.present_rate;
                },
                "discharging" => {
                    battery.charge_status = String::from("discharging");
                    battery.time_remaining = (3600 * battery.capacity_now) /
                        battery.present_rate;
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

#[derive(Debug, Default)]
pub struct Batteries {
    cells: Vec<Battery>,

    capacity_percent: f64,
    charge_percent: f64,
    charge_status: String,
    time_remaining: u64
}

// TODO figure out if should return Result or Option
impl Batteries {
    pub fn new(path: &PathBuf) -> Result<Batteries, String> {
        let mut batteries = Batteries::default();

        let entries = match path.read_dir() {
            Ok(v) => v,
            Err(_) => return Err(String::from("Error!")),
        };

        // Gather cells
        let mut lcm_capacity_design: u64 = 1;
        let mut lcm_capacity_full: u64 = 1;
        let mut charge_status_mask: u8 = 0;

        // TODO reduce the nesting here!
        // TODO remove the LCM stuff
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(entry_name) = entry.file_name().to_str() {
                    if entry_name.starts_with("BAT") {
                        // And log the error instead (?)
                        if let Some(battery) = Battery::new(&entry.path()) {
                            
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
        
        println!("Charge mask - {}", charge_status_mask);

        // TODO make charge_status be an ENUM instead!
        batteries.charge_status = String::from("unknown");
        if charge_status_mask == 2 {
            // Batteries are all full
            batteries.charge_status = String::from("full");
        } else if charge_status_mask == 1 || charge_status_mask == 3 {
            // Batteries are all charging or full
            batteries.charge_status = String::from("charging");
        } else if charge_status_mask == 4 {
            // Batteries are all discharging
            batteries.charge_status = String::from("discharging");
        }

        /*
        // TODO eliminate the LCM stuff by using floating point numbers!
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

        // TODO remove this!
        // Gather information
        let mut capacity_percent: f64 = 0.0;
        let mut charge_percent: f64 = 0.0;
        for cell in &batteries.cells {
            capacity_percent += (cell.capacity_full as f64) / (cell.capacity_design as f64);
            charge_percent += (cell.capacity_now as f64) / (cell.capacity_full as f64);
        }

        capacity_percent *= 100.0 / (batteries.cells.len() as f64);
        charge_percent *= 100.0 / (batteries.cells.len() as f64);

        println!("New capacity - {}", capacity_percent);
        println!("new charge - {}", charge_percent);
        */

        // TODO simply sum up capacity_full, capacity_design, and capacity_now
        let mut capacity_design: u64 = 0;
        let mut capacity_full: u64 = 0;
        let mut capacity_now: u64 = 0;
        for cell in &batteries.cells {
            capacity_design += cell.capacity_design;
            capacity_full += cell.capacity_full;
            capacity_now += cell.capacity_now;

            if batteries.charge_status == "charging" ||
                batteries.charge_status == "discharging" {
                batteries.time_remaining += cell.time_remaining;
            }
        }

        batteries.capacity_percent = 100.0 *
            (capacity_full as f64) / (capacity_design as f64);
        batteries.charge_percent = 100.0 * 
            (capacity_now as f64) / (capacity_full as f64);

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

fn parse_file<T>(path: &PathBuf) -> Option<T>
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

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-5;
    
    struct BatteriesTest {
        path: PathBuf,
        num_cells: usize,
        capacity_percent: f64,
        charge_percent: f64,
        charge_status: String,
        time_remaining: u64
    }
    
    fn validate_batteries(bt: &BatteriesTest) {
        let batteries = Batteries::new(&bt.path);
        assert_eq!(batteries.is_ok(), true);

        let batteries = batteries.unwrap();
        assert_eq!(batteries.cells.len(), bt.num_cells);
        relative_eq!(batteries.capacity_percent, bt.capacity_percent,
                     epsilon=EPSILON);
        relative_eq!(batteries.charge_percent, bt.charge_percent,
                     epsilon=EPSILON);
        assert_eq!(batteries.charge_status, bt.charge_status);
        assert_eq!(batteries.time_remaining, bt.time_remaining);
    }


    #[test]
    fn test_gcd_zeros() {
        assert_eq!(gcd(0, 0), 0);
        assert_eq!(gcd(0, 1), 1);
        assert_eq!(gcd(1, 0), 1);
    }

    #[test]
    fn test_gcd_ones() {
        assert_eq!(gcd(1, 1), 1);
        assert_eq!(gcd(1, 10), 1);
        assert_eq!(gcd(10, 1), 1);
        assert_eq!(gcd(7, 11), 1);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(4, 10), 2);
        assert_eq!(gcd(50, 15), 5);
        assert_eq!(gcd(49, 49), 49);
    }

    #[test]
    fn test_parse_file_u64() {
        let path = PathBuf::from("tests/one-battery/BAT0/charge_now");
        let out = parse_file::<u64>(&path);
        assert_eq!(out.is_some(), true);
        assert_eq!(out.unwrap(), 1449000);
    }

    #[test]
    fn test_parse_file_String() {
        let path = PathBuf::from("tests/one-battery/BAT0/status");
        let out = parse_file::<String>(&path);
        assert_eq!(out.is_some(), true);
        assert_eq!(out.unwrap(), String::from("Charging"));
    }

    #[test]
    fn test_battery() {
        let path = PathBuf::from("tests/one-battery/BAT0");
        let battery = Battery::new(&path);
        assert_eq!(battery.is_some(), true);

        let battery = battery.unwrap();
        assert_eq!(battery.capacity_design, 7570);
        assert_eq!(battery.capacity_full, 5394);
        assert_eq!(battery.capacity_now, 1449);
        assert_eq!(battery.charge_status, String::from("charging"));
        assert_eq!(battery.present_rate, 2643);
        assert_eq!(battery.time_remaining, 5373);
    }

    #[test]
    fn test_one_battery() {
        let bt = BatteriesTest {
            path: PathBuf::from("tests/one-battery"),
            num_cells: 1,
            capacity_percent: 100.0 * (5394.0 / 7570.0),
            charge_percent: 100.0 * (1449.0 / 5394.0),
            charge_status: String::from("charging"),
            time_remaining: 5373,
        };
        validate_batteries(&bt);
    }

    #[test]
    fn test_two_batteries_chr_chr() {
        let bt = BatteriesTest {
            path: PathBuf::from("tests/two-batteries-chr-chr"),
            num_cells: 2,
            capacity_percent: 100.0 * (11499.0 / 16320.0),
            charge_percent:  100.0 * (4640.0 / 11499.0),
            charge_status: String::from("charging"),
            time_remaining: 13867,
        };
        validate_batteries(&bt);
    }

    // TODO change the charge_status to be an Enum
    // TODO change the structure of the test below
    #[test]
    fn test_two_batteries_chr_dis() {
        let bt = BatteriesTest {
            path: PathBuf::from("tests/two-batteries-chr-dis"),
            num_cells: 2,
            capacity_percent: 100.0 * (11499.0 / 16320.0),
            charge_percent: 100.0 * (4640.0 / 11499.0),
            charge_status: String::from("unknown"),
            time_remaining: 0,
        };
        validate_batteries(&bt);
    }

    #[test]
    fn test_two_batteries_chr_ful() {
        let bt = BatteriesTest {
            path: PathBuf::from("tests/two-batteries-chr-ful"),
            num_cells: 2,
            capacity_percent: 100.0 * (11499.0 / 16320.0),
            charge_percent: 100.0 * (7554.0 / 11499.0),
            charge_status: String::from("charging"),
            time_remaining: 5373,
        };
        validate_batteries(&bt);
    }

    #[test]
    fn test_two_batteries_dis_dis() {
        let bt = BatteriesTest {
            path: PathBuf::from("tests/two-batteries-dis-dis"),
            num_cells: 2,
            capacity_percent: 100.0 * (11499.0 / 16320.0),
            charge_percent: 100.0 * (4640.0 / 11499.0),
            charge_status: String::from("discharging"),
            time_remaining: 11274,
        };
        validate_batteries(&bt);
    }

    #[test]
    fn test_two_batteries_dis_ful() {
        let bt = BatteriesTest {
            path: PathBuf::from("tests/two-batteries-dis-ful"),
            num_cells: 2,
            capacity_percent: 100.0 * (11499.0 / 16320.0),
            charge_percent: 100.0 * (7554.0 / 11499.0),
            charge_status: String::from("unknown"),
            time_remaining: 0,
        };
        validate_batteries(&bt);
    }

    #[test]
    fn test_two_batteries_ful_ful() {
        let bt = BatteriesTest {
            path: PathBuf::from("tests/two-batteries-ful-ful"),
            num_cells: 2,
            capacity_percent: 100.0 * (11499.0 / 16320.0),
            charge_percent: 100.0,
            charge_status: String::from("full"),
            time_remaining: 0,
        };
        validate_batteries(&bt);
    }
}
