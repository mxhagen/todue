#![allow(dead_code, unused)] // todo
use crate::*;

use std::cmp::Ordering::*;
use std::io;

use anyhow::{anyhow, Result};

use crossterm::{
    cursor::{self, MoveTo, RestorePosition, SavePosition},
    execute, queue,
    style::{Color, SetBackgroundColor, SetForegroundColor, Stylize},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType::All, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

/// state of the user interface
pub struct Ui<T>
where
    T: io::Write,
{
    pub active_color_pair: (Color, Color),
    pub inactive_color_pair: (Color, Color),
    pub inactive_done_color_pair: (Color, Color),
    pub header_color_pair: (Color, Color),
    pub ostream: T,
    pub scrolloff: usize,
    pub current_sort_mode: SortMode,
    pub document: Document,
    pub original_document: Document,
    pub width: usize,
    pub save_on_quit: bool,
    pub height: usize,
    pub active_entry_idx: usize,
    pub current_scroll_offset: usize,
    queue_sort_update: bool
}

impl<T> Ui<T>
where
    T: io::Write,
{
    /// returns a new `Ui` using `ostream` as its' output stream
    /// and sets up terminal state.
    pub fn init(ostream: T, document: Document) -> Self {
        let (width, height) = size().unwrap();
        let (width, height) = (width as usize, height as usize);

        let active_color_pair = (Color::Black, Color::Yellow);
        let inactive_color_pair = (Color::Reset, Color::Reset);
        let inactive_done_color_pair = (Color::DarkGrey, Color::Reset);
        let header_color_pair = (Color::Yellow, Color::Reset);

        let mut ui = Ui {
            active_color_pair,
            save_on_quit: true,
            inactive_color_pair,
            inactive_done_color_pair,
            header_color_pair,
            current_sort_mode: Default,
            active_entry_idx: 0,
            scrolloff: 8,
            current_scroll_offset: 0,
            original_document: document.clone(),
            document,
            ostream,
            width,
            height,
            queue_sort_update: false,
        };

        queue!(
            ui.ostream,
            SavePosition,
            EnterAlternateScreen,
            Clear(All),
            cursor::Hide
        );

        enable_raw_mode().unwrap_or_else(|e| {
            Log::error(format!("IO-error when enabling raw-mode: `{}`", e));
            process::exit(ErrorCode::IO.into());
        });
        ui
    }

    /// resets terminal state that `Ui::init()` sets
    pub fn deinit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.ostream,
            LeaveAlternateScreen,
            RestorePosition,
            cursor::Show
        )?;
        Ok(())
    }

    /// updates `self.width` and `self.height`.
    /// errs if the new dimensions are too small
    pub fn update_dimensions(&mut self) -> Result<()> {
        let (width, height) = size()?;
        self.width = width as usize;
        self.height = height as usize;

        if height < 4 || width < 45 {
            Err(anyhow!(
                "ui::update_dimensions: Terminal size too small to display UI"
            ))
        } else {
            Ok(())
        }
    }

    pub fn clear(&mut self) -> Result<()> {
        queue!(self.ostream, Clear(All), MoveTo(0, 0))?;
        Ok(())
    }

    /// draws the entire ui including unchanged parts
    pub fn draw(&mut self) -> Result<()> {
        self.update_dimensions()?;
        self.update_scroll_offset();
        self.clear().unwrap();

        self.apply_sort_mode();

        self.draw_header();
        for (i, entry) in self
            .document
            .entries
            .iter()
            .skip(self.current_scroll_offset)
            .enumerate()
        {
            if i >= self.inner_height() {
                break;
            }
            let mut bold = false;
            let (fg, bg) = match i == self.active_entry_idx - self.current_scroll_offset {
                true => {
                    bold = true;
                    self.active_color_pair
                }
                false if entry.done => self.inactive_done_color_pair,
                false => self.inactive_color_pair,
            };

            queue!(self.ostream, SetForegroundColor(fg), SetBackgroundColor(bg));
            let mut line = String::with_capacity(self.width);
            if entry.done {
                line += "  [x] ";
            } else {
                line += "  [ ] ";
            }

            if let Some(deadline) = entry.deadline {
                line += &format!("{}", deadline.format("(%Y-%m-%d %H:%M)"));
            } else {
                line += &" ".repeat("(YYYY-mm-dd HH:MM)".len());
            }
            line += "    ";

            let space = self.width - line.len() - 1;
            let mut text = entry.text.clone();
            if text.chars().count() > space {
                text = text[0..space - 3].to_string() + "... ";
            }
            line += &text;
            let space = self.width - line.len();
            line += &" ".repeat(space);

            match bold {
                true => write!(self.ostream, "{}\r\n", line.bold()),
                false => write!(self.ostream, "{}\r\n", line),
            };
            queue!(
                self.ostream,
                SetBackgroundColor(Color::Reset),
                SetForegroundColor(Color::Reset)
            );
        }

        self.ostream.flush()?;
        Ok(())
    }

    pub fn draw_header(&mut self) {
        let (fg, bg) = self.header_color_pair;
        queue!(self.ostream, SetForegroundColor(fg), SetBackgroundColor(bg));

        let mut line = String::with_capacity(self.width);
        line += "  [todue] ";
        line += &" ".repeat("  [x] (YYYY-mm-dd HH:MM)    ".len() - line.len());
        line += &self.document.title.clone().unwrap_or("TODO".into());
        let space = self.width - line.len();
        line += &" ".repeat(space);
        write!(self.ostream, "{}\r\n", line);
        write!(self.ostream, "{}\r\n", "—".repeat(self.width));
        queue!(
            self.ostream,
            SetForegroundColor(Color::Reset),
            SetBackgroundColor(Color::Reset)
        );
    }

    /// update the index of the first *shown* entry using `self.scrolloff`
    pub fn update_scroll_offset(&mut self) {
        if self.current_scroll_offset + self.scrolloff >= self.active_entry_idx {
            let diff = (self.current_scroll_offset + self.scrolloff).abs_diff(self.active_entry_idx);
            self.current_scroll_offset = self.current_scroll_offset.saturating_sub(diff);

        } else if (self.current_scroll_offset + self.inner_height()).saturating_sub(self.scrolloff) <= self.active_entry_idx {
            let diff = (self.current_scroll_offset + self.inner_height()).saturating_sub(self.scrolloff).abs_diff(self.active_entry_idx);
            self.current_scroll_offset = (self.current_scroll_offset + diff).min(
                self.document
                    .entries
                    .len()
                    .saturating_sub(self.inner_height()),
            );
        }
    }

    /// get height in characters excluding header/title
    pub fn inner_height(&self) -> usize {
        self.height - 3
    }

    pub fn move_selection(&mut self, dir: MoveDirection) -> Result<()> {
        match dir {
            Down => {
                self.active_entry_idx = (self.active_entry_idx + 1) % self.document.entries.len()
            }
            Up => {
                self.active_entry_idx = ((self.active_entry_idx as isize - 1)
                    .rem_euclid(self.document.entries.len() as isize))
                    as usize
            }
        }
        Ok(())
    }

    pub fn move_selection_to_bottom(&mut self) {
        self.active_entry_idx = self.document.entries.len() - 1;
    }

    pub fn move_selection_to_top(&mut self) {
        self.active_entry_idx = 0;
    }

    pub fn move_selected_entry(&mut self, dir: MoveDirection) {
        let swap_idx = match dir {
            Down => (self.active_entry_idx + 1) % self.document.entries.len(),
            Up => {
                ((self.active_entry_idx as isize - 1)
                    .rem_euclid(self.document.entries.len() as isize)) as usize
            }
        };
        self.document.entries.swap(self.active_entry_idx, swap_idx);
        self.move_selection(dir);
    }

    pub fn toggle_active_entry(&mut self) {
        let state = self.document.entries[self.active_entry_idx].done;
        self.document.entries[self.active_entry_idx].done = !state;
    }

    pub fn cycle_sort_mode(&mut self) {
        self.current_sort_mode = match self.current_sort_mode {
            Default => ByDeadlineDescending,
            ByDeadlineDescending => ByDeadlineAscending,
            ByDeadlineAscending => ByTextAscending,
            ByTextAscending => ByTextDescending,
            ByTextDescending => Default,
        };
        self.queue_sort_update = true;
    }

    pub fn apply_sort_mode(&mut self) {
        if self.queue_sort_update {
            match self.current_sort_mode {
                Default => self.document = self.original_document.clone(),
                ByDeadlineDescending => {
                    self.document.entries.sort_by_key(|entry| entry.deadline);
                    self.document.entries.reverse();
                },
                ByDeadlineAscending => self.document.entries.sort_by_key(|entry| entry.deadline),
                ByTextAscending => self.document.entries.sort_by_key(|entry| entry.text.to_lowercase()),
                ByTextDescending => {
                    self.document.entries.sort_by_key(|entry| entry.text.to_lowercase());
                    self.document.entries.reverse();
                },
            }
            self.queue_sort_update = false;
        }
    }

    pub fn dont_save_on_quit(&mut self) {
        self.save_on_quit = false;
        Log::info("Quitting without saving file...");
    }
}

pub enum MoveDirection {
    Down,
    Up,
}
pub use MoveDirection::*;

pub enum SortMode {
    Default,
    ByDeadlineDescending,
    ByDeadlineAscending,
    ByTextAscending,
    ByTextDescending,
}
pub use SortMode::*;
