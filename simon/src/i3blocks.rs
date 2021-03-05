use std::fmt;

use serde::{Deserialize, Serialize};

pub const IO_PAUSE: std::time::Duration = std::time::Duration::from_millis(10);

#[derive(Serialize, Debug)]
pub struct I3BlocksOutput {
    pub full_text: String,
}

impl fmt::Display for I3BlocksOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = serde_json::to_string(&self).unwrap();
        write!(f, "{}", text)
    }
}

#[derive(Deserialize, Debug)]
pub struct I3ClickEvent {
    pub name: String,
    pub button: i32,
    pub relative_x: i32,
    pub width: i32,
}
