use std::cell::RefCell;
use std::rc::Rc;

use crate::i3blocks::{I3BlocksOutput, I3ClickEvent};

pub enum MenuAction {
    NextItem,
    PreviousItem,
    Exit,
    Noop,
}

pub trait MenuItem {
    fn to_output(&mut self) -> I3BlocksOutput;
    fn handle_click(&mut self, click: &I3ClickEvent) -> MenuAction;
}

pub trait SelectItemOption {
    fn to_string(&self) -> String;
}

pub struct ButtonItem {
    pub label: &'static str,
}

impl MenuItem for ButtonItem {
    fn to_output(&mut self) -> I3BlocksOutput {
        I3BlocksOutput {
            full_text: String::from(self.label),
        }
    }

    fn handle_click(&mut self, click: &I3ClickEvent) -> MenuAction {
        match click.button {
            1 => MenuAction::Exit,
            4 => MenuAction::NextItem,
            5 => MenuAction::PreviousItem,
            _ => MenuAction::Noop,
        }
    }
}

pub struct SelectItem<T>
where
    T: SelectItemOption + Copy,
{
    pub external: Rc<RefCell<T>>,
    pub label: &'static str,
    pub options: Vec<T>,
    pub index: usize,
}

impl<T> MenuItem for SelectItem<T>
where
    T: SelectItemOption + Copy,
{
    fn to_output(&mut self) -> I3BlocksOutput {
        let option = self.options[self.index].to_string();
        let full_text = format!("{} {}", self.label, option);
        I3BlocksOutput { full_text }
    }

    fn handle_click(&mut self, click: &I3ClickEvent) -> MenuAction {
        match click.button {
            1 => {
                let length = self.options.len();
                self.index = (self.index + 1).rem_euclid(length);
                *self.external.borrow_mut() = self.options[self.index];
                MenuAction::Noop
            }
            3 => {
                let length = self.options.len() as isize;
                self.index = ((self.index as isize) - 1).rem_euclid(length) as usize;
                *self.external.borrow_mut() = self.options[self.index];
                MenuAction::Noop
            }
            4 => MenuAction::NextItem,
            5 => MenuAction::PreviousItem,
            _ => MenuAction::Noop,
        }
    }
}

pub struct Menu {
    items: Vec<Box<dyn MenuItem>>,
    index: usize,
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            items: Vec::new(),
            index: 0,
        }
    }

    pub fn add_menu_item(&mut self, menu_item: Box<dyn MenuItem>) {
        self.items.push(menu_item);
    }

    pub fn interact(&mut self) {
        loop {
            let item = &mut self.items[self.index];
            println!("{}", item.to_output());

            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).unwrap();
            let click: I3ClickEvent = serde_json::from_str(&buffer).unwrap();

            let menu_action = item.handle_click(&click);
            match menu_action {
                MenuAction::NextItem => {
                    let length = self.items.len();
                    self.index = (self.index + 1).rem_euclid(length);
                }
                MenuAction::PreviousItem => {
                    let length = self.items.len() as isize;
                    self.index = ((self.index as isize) - 1).rem_euclid(length) as usize
                }
                MenuAction::Exit => break,
                MenuAction::Noop => (),
            }
        }
    }
}
