use battery;
use battery::I3Block;

fn main() {
    // Parse config file
    let args: Vec<String> = std::env::args().collect();
    let config = match battery::Config::new(&args) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            std::process::exit(0);
        }
    };

    // Parse batteries
    let path = std::path::PathBuf::from("/sys/class/power_supply");
    match battery::Batteries::new(&path, config.log_batteries) {
        Ok(batteries) => println!("{}", batteries.format_i3()),
        Err(e) => {
            println!("{}", e);
            std::process::exit(0);
        }
    }
}
