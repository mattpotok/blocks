use std::fmt;
use std::path::PathBuf;

use log::{error, info, warn};
use serde::Deserialize;
use serde_yaml;
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};

// Constants
const DEFAULT_ERROR: &str = "BAT Error!\nBAT Error!\n#FF0000";

// Traits
pub trait I3Block{
    fn format_i3(&self) -> String;
}

/// Container for configuration options
#[derive(Deserialize)]
pub struct Config {
    /// Path to the log file
    log_file_path: PathBuf,

    /// Log battery information
    #[serde(default = "Config::default_bool_false")]
    pub log_batteries: bool,
}

impl Config {
    /// Parses configuration options
    /// 
    /// The constructor additionally initialize a `WriteLogger` to log any
    /// errors to a log file. Logged errors will contain a detailed 
    /// description about the failure that can't be captured by the `battery`
    /// block in the i3 bar.
    /// 
    /// # Arguments
    ///
    /// - `args`: A slice of command line arguments
    ///
    /// # Returns
    ///
    /// A `Result`:
    /// - `Ok`: A `Config` with parsed configuration options
    /// - `Err`: A `String` with error to be displayed by i3
    pub fn new(args: &[String]) -> Result<Config, String> {
        // Check number of arguments
        if args.len() != 2 {
            let e = "BAT Args error!\nBAT Args error!\n#FF0000";
            return Err(e.into());
        }

        // Open configuration file
        let file = match std::fs::File::open(&args[1]) {
            Ok(v) => v,
            Err(_) => {
                let e = "BAT File error!\nBAT File error!\n#FF0000";
                return Err(e.into());
            },
        };

        // Parse configuration file
        let config: Config = match serde_yaml::from_reader(file) {
            Ok(v) => v,
            Err(_) => {
                let e = "BAT Parse error!\nBAT Parse error!\n#FF0000";
                return Err(e.into());
            },
        };

        // Create logger
        let file = match std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&config.log_file_path) {
                Ok(v) => v,
                Err(_) => {
                    let e = "BAT File error!\nBAT File error!\n#FF0000";
                    return Err(e.into());
                },
            };

        let logger = ConfigBuilder::new()
            .set_time_format_str("%a %b %e %T %Y")
            .set_time_to_local(true)
            .build();

        if let Err(_) = WriteLogger::init(LevelFilter::Info, logger, file) {
            let e = "BAT Logger error!\nBAT Logger error!\n#FF0000";
            return Err(e.into());
        }

        Ok(config)
    }

    fn default_bool_false() -> bool {
        false
    }
}

/// Battery charge states
#[derive(Debug, PartialEq)]
enum ChargeStatus {
    Charging,
    Discharging,
    Full,
    Unknown,
}

impl Default for ChargeStatus {
    fn default() -> Self { ChargeStatus::Unknown } 
}

impl fmt::Display for ChargeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ChargeStatus::Charging => write!(f, "charging"),
            ChargeStatus::Discharging => write!(f, "discharging"),
            ChargeStatus::Full => write!(f, "full"),
            ChargeStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Battery information
#[derive(Default)]
pub struct Battery {
    name: String,                     // name of the cell

    charge_full: Option<u64>,         // last 'full' charge (mAh)
    charge_full_design: Option<u64>,  // 'full' design charge (mAh)
    charge_now: Option<u64>,          // present charge (mAh)
    current_now: Option<u64>,         // present current (mA)
    status: Option<String>,           // charging status

    capacity_design: u64,             // 'full' design capacity (mAh)
    capacity_full: u64,               // last 'full' capacity (mAh)
    capacity_now: u64,                // present capacity (mAh)
    charge_status: ChargeStatus,      // charging status
    present_rate: u64,                // present current (mA)
    time_remaining: u64,              // remaining (dis)charge time (s)
}

impl Battery {
    pub fn new(path: &PathBuf) -> Option<Battery> {
        let mut battery = Battery::default();

        // Get battery name
        match path.file_name() {
            Some(v) => {
                match v.to_str() {
                    Some(name) => battery.name = String::from(name),
                    None => {
                        error!("battery::Battery::new unable to parse name");
                        return None;
                    },
                }
            },
            None => {
                error!("battery::Battery::new unable to parse name");
                return None;
            },
        };
        
        // Read battery directory
        let entries = match path.read_dir() {
            Ok(v) => v,
            Err(e) => {
                error!("battery::Battery::new {}", e);
                return None;
            },
        };

        // Parse files
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(entry_name) = entry.file_name().to_str() {
                    let entry_path = entry.path();
                    match entry_name {
                        "charge_full" => battery.charge_full =
                            parse_file::<u64>(&entry_path).map(|v| v / 1000),
                        "charge_full_design" => battery.charge_full_design =
                            parse_file::<u64>(&entry_path).map(|v| v / 1000),
                        "charge_now" => battery.charge_now =
                            parse_file::<u64>(&entry_path).map(|v| v / 1000),
                        "current_now" => battery.current_now =
                            parse_file::<u64>(&entry_path).map(|v| v / 1000),
                        "status" => battery.status = parse_file(&entry_path),
                        _ => (),
                    }
                }
            }
        }

        // Determine battery's current capacity
        if battery.charge_full.is_some() {
            battery.capacity_full = battery.charge_full.unwrap();
        } else {
            error!("battery::Battery::new: unable to compute `capacity_full`");
            return None;
        }

        // Determine battery's design capacity
        if battery.charge_full_design.is_some() {
            battery.capacity_design = battery.charge_full_design.unwrap();
        } else {
            error!("battery::Battery::new:\
                    unable to compute `capacity_design`");
            return None;
        }

        // Determine battery's present capacity
        if battery.charge_now.is_some() {
            battery.capacity_now = battery.charge_now.unwrap();
        } else {
            error!("battery::Battery::new: unable to compute `capacity_now`");
            return None;
        }

        // Determine battery's (dis)charging rate
        if battery.current_now.is_some() {
            battery.present_rate = battery.current_now.unwrap();
        } else {
            error!("battery::Battery::new unable to compute `current_now`");
            return None;
        }

        // Determine battery's state
        if let Some(status) = battery.status.clone() {
            match status.as_str() {
                "Charging" => {
                    battery.charge_status = ChargeStatus::Charging;
                    battery.time_remaining = (3600 *
                        (battery.capacity_full - battery.capacity_now)) /
                        battery.present_rate;
                },
                "Discharging" => {
                    battery.charge_status = ChargeStatus::Discharging;
                    battery.time_remaining = (3600 * battery.capacity_now) /
                        battery.present_rate;
                },
                "Full" => battery.charge_status = ChargeStatus::Full,
                _ => {}
            };
        }

        // Warn and cap on invalid battery reading
        if battery.capacity_now > battery.capacity_full {
            battery.capacity_now = battery.capacity_full;
            warn!("battery::Battery::new invalid reading `capacity_now` >\
                  `capacity_full`, capping member");
        }

        Some(battery)
    }
}

#[derive(Default)]
pub struct Batteries {
    cells: Vec<Battery>,          // available individual battery cells

    capacity_percent: f64,        // current capacity (%)
    charge_percent: f64,          // current charge (%)
    charge_status: ChargeStatus,  // overall charging status
    time_remaining: u64           // remaining (dis)charge times (s)
}

impl Batteries {
    pub fn new(path: &PathBuf, log: bool) -> Result<Batteries, String> {
        let mut batteries = Batteries::default();

        // Read `power_supply` directory
        let entries = match path.read_dir() {
            Ok(v) => v,
            Err(e) => {
                error!("battery::Batteries::new: {}", e);
                return Err(DEFAULT_ERROR.into());
            },
        };

        // Gather all battery cells
        let mut charge_mask: u8 = 0;
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(entry_name) = entry.file_name().to_str() {
                    if !entry_name.starts_with("BAT") {
                        continue;
                    }

                    // Parse a battery
                    if let Some(battery) = Battery::new(&entry.path()) {
                        match battery.charge_status {
                            ChargeStatus::Charging => charge_mask |= 1,
                            ChargeStatus::Discharging => charge_mask |= 2,
                            ChargeStatus::Full => charge_mask |= 4,
                            _ => charge_mask |= 8,
                        }
                        batteries.cells.push(battery);
                    } else {
                        error!("battery::Batteries::new:\
                                unable to parse battery {}", entry_name);
                    }
                }
            }
        }

        // Check if any cells
        if batteries.cells.len() == 0 {
            error!("battery::Batteries::new: no battery cells");
            return Err(DEFAULT_ERROR.into());
        }

        // Determine overall charge status
        if charge_mask == 4 {
            // Batteries are all full
            batteries.charge_status = ChargeStatus::Full;
        } else if charge_mask == 1 || charge_mask == 5 {
            // Batteries are all charging or full
            batteries.charge_status = ChargeStatus::Charging;
        } else if charge_mask == 2 {
            // Batteries are all discharging
            batteries.charge_status = ChargeStatus::Discharging;
        }

        // Compute overall information
        let mut capacity_design: u64 = 0;
        let mut capacity_full: u64 = 0;
        let mut capacity_now: u64 = 0;
        for cell in &batteries.cells {
            capacity_design += cell.capacity_design;
            capacity_full += cell.capacity_full;
            capacity_now += cell.capacity_now;

            if batteries.charge_status == ChargeStatus::Charging ||
                batteries.charge_status == ChargeStatus::Discharging {
                batteries.time_remaining += cell.time_remaining;
            }

            if log {
                info!("battery::Batteries::new: cell {}\n\
                      \t* capacity design - {} mAh\n\
                      \t* capacity full - {} mAh\n\
                      \t* capacity now - {} mAh\n\
                      \t* charge status - {}",
                      cell.name, cell.capacity_design, cell.capacity_full,
                      cell.capacity_now, cell.charge_status.to_string());
            }
        }

        if log {
            info!("battery::Batteries::new: all cells\n\
                  \t* capacity design - {} mAh\n\
                  \t* capacity full - {} mAh\n\
                  \t* capacity now - {} mAh\n\
                  \t* charge status - {}",
                  capacity_design, capacity_full, capacity_now,
                  batteries.charge_status.to_string());
        }

        // Compute percentages
        batteries.capacity_percent = 100.0 *
            (capacity_full as f64) / (capacity_design as f64);
        batteries.charge_percent = 100.0 * 
            (capacity_now as f64) / (capacity_full as f64);

        Ok(batteries)
    }
}

impl I3Block for Batteries {
    fn format_i3(&self) -> String {
        let status = match self.charge_status {
            ChargeStatus::Charging => String::from("CHR"),
            ChargeStatus::Discharging => String::from("DIS"),
            ChargeStatus::Full => String::from("FUL"),
            ChargeStatus::Unknown => String::from("UKN"),
        };

        let color = match self.charge_status {
            ChargeStatus::Charging => "#00FF00",
            ChargeStatus::Discharging => {
                let charge = self.charge_percent;
                if charge >= 90.0 {
                    "#00FF00"
                } else if charge >= 80.0 {
                    "#37FF00"
                } else if charge >= 70.0 {
                    "#75FF00"
                } else if charge >= 60.0 {
                    "#AAFF00"
                } else if charge >= 50.0 {
                    "#E1FF00"
                } else if charge >= 40.0 {
                    "#FFE100"
                } else if charge >= 30.0 {
                    "#FFAA00"
                } else if charge >= 20.0 {
                    "#FF7300"
                } else if charge >= 10.0 {
                    "#FF3700"
                } else {
                    "#FF000"
                }
            },
            _ => "#FFFFFF",
        };

        let time = match self.charge_status {
            ChargeStatus::Charging | ChargeStatus::Discharging => {
                let mut time = self.time_remaining; 
                let hours = time / 3600;

                time = time - (hours * 3600);
                let minutes = time / 60;

                time = time - (minutes * 60);
                let seconds = time;

                format!(" {}:{:02}:{:02}", hours, minutes, seconds)
            },
            _ => String::from(""),
        };

        let full_text = format!("BAT {:.0}% ({:.0}%) {}{}", self.charge_percent,
                                self.capacity_percent, status, time);
        let short_text = format!("BAT {:.0}%{}", self.charge_percent, time);

        format!("{}\n{}\n{}", full_text, short_text, color)
    }
}

fn parse_file<T>(path: &PathBuf) -> Option<T>
        where T: std::str::FromStr + std::fmt::Debug {
    let contents = match std::fs::read_to_string(path) {
        Ok(v) => v,
        Err(_) => return None,
    };

    match contents.trim().parse::<T>() {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;

    const EPSILON: f64 = 1e-5;
    
    struct BatteriesTest {
        path: PathBuf,
        num_cells: usize,
        capacity_percent: f64,
        charge_percent: f64,
        charge_status: ChargeStatus,
        time_remaining: u64
    }
    
    fn validate_batteries(bt: &BatteriesTest) {
        let batteries = Batteries::new(&bt.path, false);
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
    fn test_parse_file_u64() {
        let path = PathBuf::from("tests/one-battery/BAT0/charge_now");
        let out = parse_file::<u64>(&path);
        assert_eq!(out.is_some(), true);
        assert_eq!(out.unwrap(), 1449000);
    }

    #[test]
    fn test_parse_file_string() {
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
        assert_eq!(battery.charge_status, ChargeStatus::Charging);
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
            charge_status: ChargeStatus::Charging,
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
            charge_status: ChargeStatus::Charging,
            time_remaining: 13867,
        };
        validate_batteries(&bt);
    }

    #[test]
    fn test_two_batteries_chr_dis() {
        let bt = BatteriesTest {
            path: PathBuf::from("tests/two-batteries-chr-dis"),
            num_cells: 2,
            capacity_percent: 100.0 * (11499.0 / 16320.0),
            charge_percent: 100.0 * (4640.0 / 11499.0),
            charge_status: ChargeStatus::Unknown,
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
            charge_status: ChargeStatus::Charging,
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
            charge_status: ChargeStatus::Discharging,
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
            charge_status: ChargeStatus::Unknown,
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
            charge_status: ChargeStatus::Full,
            time_remaining: 0,
        };
        validate_batteries(&bt);
    }

    #[test]
    fn test_one_battery_format_i3() {
        let output = String::from(
            "BAT 27% (71%) CHR 1:29:33\nBAT 27% 1:29:33\n#00FF00"
        );

        let path = PathBuf::from("tests/one-battery");
        let batteries = Batteries::new(&path, false);
        assert_eq!(batteries.is_ok(), true);

        let batteries = batteries.unwrap();
        assert_eq!(output, batteries.format_i3());
    }
}
