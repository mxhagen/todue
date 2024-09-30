#![allow(dead_code, unused)] // todo
use crate::*;

use std::collections::HashMap;

use anyhow::*;
use crossterm::event::KeyEvent;

/// Mode of the TUI
#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum Mode {
    #[default]
    Normal,
    Insert(EditMode),
    Datetime,
    Visual,
}

/// Mode for the line editor
#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum EditMode {
    #[default]
    Normal,
    Insert,
    Visual,
    Replace,
}

type KeymapCallback = Box<for<'a> fn(&'a mut App)>;

#[derive(Debug)]
pub struct Keymap {
    pub map: HashMap<(Mode, KeyEvent), KeymapCallback>,
}

impl Keymap {
    pub fn handle(&self, key: KeyEvent, app: &mut App) -> anyhow::Result<()> {
        self.map
            .get(&(app.mode.clone(), key))
            .map(|f| f(app))
            .ok_or(anyhow!("Associated mapping not found."))
    }

    pub fn register(&mut self, mode: Mode, key: KeyEvent, f: KeymapCallback) {
        self.map.insert((mode, key), f);
    }
}

impl Default for Keymap {
    fn default() -> Self {
        let mut map = Self {
            map: HashMap::new(),
        };
        use Mode::*;
        map.register(Normal, Char('q').into(), Box::new(App::quit));
        map.register(
            Normal,
            Char('Q').into(),
            Box::new(|app: &mut App| {
                app.ui.dont_save_on_quit();
                app.quit()
            }),
        );
        map.register(
            Normal,
            Char('j').into(),
            Box::new(|app: &mut App| app.ui.move_selection(Down).unwrap()),
        );
        map.register(
            Normal,
            Char('k').into(),
            Box::new(|app: &mut App| app.ui.move_selection(Up).unwrap()),
        );
        map.register(
            Normal,
            Char('J').into(),
            Box::new(|app: &mut App| app.ui.move_selected_entry(Down)),
        );
        map.register(
            Normal,
            Char('K').into(),
            Box::new(|app: &mut App| app.ui.move_selected_entry(Up)),
        );
        map.register(
            Normal,
            Char('G').into(),
            Box::new(|app: &mut App| app.ui.move_selection_to_top()),
        );
        map.register(
            Normal,
            Char('g').into(),
            Box::new(|app: &mut App| app.ui.move_selection_to_bottom()),
        );
        map.register(
            Normal,
            Char(' ').into(),
            Box::new(|app: &mut App| app.ui.toggle_active_entry()),
        );
        map.register(
            Normal,
            Char('s').into(),
            Box::new(|app: &mut App| app.ui.cycle_sort_mode()),
        );
        map.register(
            Normal,
            Char('-').into(),
            Box::new(|app: &mut App| {
                Log::info("Debug panic keybind '-' invoked -- panicking...");
                panic!();
            }),
        );
        map
    }
}
