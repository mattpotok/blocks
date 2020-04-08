use std::path::PathBuf;

use battery;
use battery::I3Block;

fn main() {
    // Parse config file
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = battery::Config::new(&args) {
        println!("{}", e);
        std::process::exit(1);
    }

    // Parse batteries
    let path = PathBuf::from("/sys/class/power_supply");
    match battery::Batteries::new(&path) {
        Ok(batteries) => println!("{}", batteries.format_i3()),
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        },
    }
}
