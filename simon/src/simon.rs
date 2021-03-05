// Notes
// - It doesn't seem possible to drain STDIN of existing input so any extraneous
//   clicks during the sequence display will be processed immediately afterwards
// - Sound wasn't working as expected so it was excluded for the time being

use log::info;
use std::cell::RefCell;
use std::fmt;
use std::io::BufRead;
use std::path::PathBuf;
use std::rc::Rc;
use std::{thread, time};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use serde::Deserialize;
use serde_yaml;
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};

use crate::i3blocks::*;
use crate::menu::*;

#[derive(Clone, Copy, PartialEq)]
enum Turns {
    Eight = 8,
    Fourteen = 14,
    Twenty = 20,
    ThirtyOne = 31,
    Infinity = -1,
}

impl fmt::Display for Turns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Turns::Eight => write!(f, "8"),
            Turns::Fourteen => write!(f, "14"),
            Turns::Twenty => write!(f, "20"),
            Turns::ThirtyOne => write!(f, "31"),
            Turns::Infinity => write!(f, "∞"),
        }
    }
}

impl SelectItemOption for Turns {
    fn to_string(&self) -> String {
        std::string::ToString::to_string(&self)
    }
}

impl SelectItemOption for bool {
    fn to_string(&self) -> String {
        match &self {
            false => "N".into(),
            true => "Y".into(),
        }
    }
}

#[derive(Deserialize)]
pub struct Configuration {
    log_file_path: PathBuf,
}

impl Configuration {
    pub fn new(args: &[String]) -> Result<Configuration, I3BlocksOutput> {
        const ERROR_TEXT: &str = "<span foreground=\"#FF0000\">E: {}</span>";

        let mut error = I3BlocksOutput {
            full_text: "".into(),
        };

        if args.len() != 2 {
            error.full_text = ERROR_TEXT.replace("{}", "# arg");
            return Err(error);
        }

        // Open configuration file
        let file = match std::fs::File::open(&args[1]) {
            Ok(v) => v,
            Err(_) => {
                error.full_text = ERROR_TEXT.replace("{}", "cfg /");
                return Err(error);
            }
        };

        // Parse configuration file
        let config: Configuration = match serde_yaml::from_reader(file) {
            Ok(v) => v,
            Err(_) => {
                error.full_text = ERROR_TEXT.replace("{}", "parse");
                return Err(error);
            }
        };

        // Create logger
        let file = match std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&config.log_file_path)
        {
            Ok(v) => v,
            Err(_) => {
                error.full_text = ERROR_TEXT.replace("{}", "log /");
                return Err(error);
            }
        };

        let logger = ConfigBuilder::new()
            .set_time_format_str("%a %b %e %T %Y")
            .set_time_to_local(true)
            .build();

        if let Err(_) = WriteLogger::init(LevelFilter::Info, logger, file) {
            error.full_text = ERROR_TEXT.replace("{}", "log");
            return Err(error);
        }

        Ok(config)
    }
}

struct Button<'a> {
    pub color: Color,
    pub on: &'a str,
    pub off: &'a str,
}

#[derive(Debug, PartialEq)]
enum Color {
    Green,
    Red,
    Blue,
    Yellow,
    None,
}

impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        match rng.gen_range(0..=3) {
            0 => Color::Green,
            1 => Color::Red,
            2 => Color::Blue,
            3 => Color::Yellow,
            _ => Color::None,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Color::Green => write!(f, "green"),
            Color::Red => write!(f, "red"),
            Color::Blue => write!(f, "blue"),
            Color::Yellow => write!(f, "yellow"),
            Color::None => write!(f, "none"),
        }
    }
}

pub struct Simon {
    // Game state
    sequence: Vec<Color>,

    // Menu
    menu: Menu,

    // User settings
    cheat: Rc<RefCell<bool>>,
    turns: Rc<RefCell<Turns>>,
}

impl Simon {
    pub fn new(args: &[String]) -> Result<Simon, I3BlocksOutput> {
        if let Err(e) = Configuration::new(&args) {
            return Err(e);
        }

        let mut simon = Simon {
            sequence: Vec::<Color>::new(),

            menu: Menu::new(),

            cheat: Rc::new(RefCell::new(false)),
            turns: Rc::new(RefCell::new(Turns::Eight)),
        };

        let start_button = Box::new(ButtonItem { label: "Simon!" });
        simon.menu.add_menu_item(start_button);

        let cheat_select = Box::new(SelectItem {
            external: Rc::clone(&simon.cheat),
            label: "Cheat",
            options: vec![false, true],
            index: *simon.cheat.borrow() as usize,
        });
        simon.menu.add_menu_item(cheat_select);

        let turns_select = Box::new(SelectItem {
            external: Rc::clone(&simon.turns),
            label: "Turns",
            options: vec![
                Turns::Eight,
                Turns::Fourteen,
                Turns::Twenty,
                Turns::ThirtyOne,
                Turns::Infinity,
            ],
            index: match *simon.turns.borrow() {
                Turns::Eight => 0,
                Turns::Fourteen => 1,
                Turns::Twenty => 2,
                Turns::ThirtyOne => 3,
                Turns::Infinity => 4,
            },
        });
        simon.menu.add_menu_item(turns_select);

        return Ok(simon);
    }

    pub fn play(&mut self) {
        // Constants
        const DEFEAT: &str = "{\"full_text\": \"Defeat!\"}";
        const VICTORY: &str = "{\"full_text\": \"Victory!\"}";
        const TURN_PAUSE: time::Duration = time::Duration::from_millis(800);

        loop {
            self.menu.interact();

            // Display empty board
            self.display_buttons(&Color::None);

            let turns = *self.turns.borrow() as usize;
            self.sequence.clear();
            'game: for turn in 0..turns {
                let random_color: Color = rand::random();
                self.sequence.push(random_color);

                if *self.cheat.borrow() == true {
                    let sequence_str = self
                        .sequence
                        .iter()
                        .map(|color| color.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");

                    info!("Turn {} - [{}]", turn, sequence_str);
                }

                thread::sleep(TURN_PAUSE); // Required to display empty board
                self.show_sequence();

                for color in self.sequence.iter() {
                    let guess_color = self.get_pressed_button();
                    if guess_color != *color {
                        info!("Defeat - turns {}!", turn);
                        println!("{}", DEFEAT);
                        break 'game;
                    }
                }

                if self.sequence.len() == (turns as usize) {
                    info!("Victory - turns {}!", turn);
                    println!("{}", VICTORY);
                }
            }

            // Display defeat/victory message
            self.wait_for_click();
        }
    }

    fn display_buttons(&self, color: &Color) {
        const BUTTONS: [Button; 4] = [
            Button {
                color: Color::Green,
                on: "00FF00",
                off: "93C47D",
            },
            Button {
                color: Color::Red,
                on: "FF0000",
                off: "E06666",
            },
            Button {
                color: Color::Blue,
                on: "0000FF",
                off: "6FA8DE",
            },
            Button {
                color: Color::Yellow,
                on: "FF9900",
                off: "F6B26B",
            },
        ];

        const BUTTON_TEXT: &str = "<span foreground=\"#{}\">██</span>";

        let buttons = BUTTONS
            .iter()
            .map(|button| {
                if button.color == *color {
                    BUTTON_TEXT.replace("{}", button.on)
                } else {
                    BUTTON_TEXT.replace("{}", button.off)
                }
            })
            .collect::<Vec<String>>()
            .join("");

        let output = I3BlocksOutput { full_text: buttons };
        let output = serde_json::to_string(&output).unwrap();
        println!("{}", output);
    }

    fn get_pressed_button(&self) -> Color {
        const PULSE_MS: time::Duration = time::Duration::from_millis(250);

        // TODO return the time taken to press the button
        // if it takes longer than 1.5 seconds, return Color::NONE to end game
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        let click: I3ClickEvent = serde_json::from_str(&buffer).unwrap();

        let button_width = click.width / 4;
        let button = click.relative_x / button_width;

        let color = match button {
            0 => Color::Green,
            1 => Color::Red,
            2 => Color::Blue,
            3 => Color::Yellow,
            _ => Color::None,
        };

        self.display_buttons(&color);
        thread::sleep(PULSE_MS);
        self.display_buttons(&Color::None);
        thread::sleep(IO_PAUSE); // Required to display defeat/victory

        return color;
    }

    fn show_sequence(&self) {
        // Note: doubled this value to make it easier to distinguish
        const OFF_PERIOD: time::Duration = time::Duration::from_millis(100);

        if self.sequence.len() == 0 {
            return;
        }

        let on_period = match self.sequence.len() {
            1..=5 => time::Duration::from_millis(420),
            6..=13 => time::Duration::from_millis(320),
            14..=31 => time::Duration::from_millis(220),
            _ => time::Duration::from_millis(170),
        };

        for color in self.sequence.iter() {
            self.display_buttons(color);
            thread::sleep(on_period);
            self.display_buttons(&Color::None);
            thread::sleep(OFF_PERIOD);
        }
    }

    fn wait_for_click(&self) {
        let mut buffer = String::new();
        let stdin = std::io::stdin();
        stdin.lock().read_line(&mut buffer).unwrap();
    }
}
