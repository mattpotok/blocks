use std::{thread, time};

mod i3blocks;
mod menu;
mod simon;

const I3BLOCKS_PAUSE: time::Duration = time::Duration::from_millis(1000);

fn main() {
    // Wait for i3blocks to initialize
    thread::sleep(I3BLOCKS_PAUSE);

    // Parse configuration for Simon
    let args: Vec<String> = std::env::args().collect();
    let mut simon = match simon::Simon::new(&args) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            thread::sleep(i3blocks::IO_PAUSE);
            return;
        }
    };

    simon.play();
}
