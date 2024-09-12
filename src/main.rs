use std::{
    fs,
    io::{self, Write},
    path, process, time,
};

use crossterm::event::{poll, read, Event::*, KeyCode::Char};

mod md;
use md::*;

mod ui;
use ui::*;

mod cli;

mod log;
pub use log::*;

mod tests;

fn main() -> anyhow::Result<()> {
    let command = cli::new();
    let args = command.clone().get_matches();

    let md_file = match args.get_one::<String>("file") {
        Some(path) => path,
        _ => {
            Log::warn("No filename specified -- falling back to `todo.md`");
            &"todo.md".to_string()
        }
    };

    if !path::Path::new(md_file).exists() {
        Log::error_exit_with(
            ErrorCode::IO,
            format!("Markdown file `{md_file}` not found. Exiting..."),
        );
    }

    let md = fs::read_to_string(md_file).unwrap();
    let document = Document::from_md(md)?;

    let mut ui = Ui::init(io::stdout(), document);
    let _guard = DropGuard {
        exec_on_drop: || {
            {
                let _ = Ui::init(&mut io::stdout(), Document::default()).deinit();
            }
            .into()
        },
    };

    loop {
        ui.draw()?;
        if poll(time::Duration::from_millis(250)).unwrap_or(false) {
            if let Ok(Key(k)) = read() {
                match k.code {
                    Char('q') => break,
                    Char('j') => ui.move_selection(Down)?,
                    Char('k') => ui.move_selection(Up)?,
                    Char('J') => ui.move_selected_entry(Down),
                    Char('K') => ui.move_selected_entry(Up),
                    Char('G') => ui.move_selection_to_bottom(),
                    Char('g') => ui.move_selection_to_top(),
                    Char(' ') => ui.toggle_active_entry(),
                    Char('s') => ui.cycle_sort_mode(),
                    Char('Q') => { ui.dont_save_on_quit(); break; },
                    _ => {}
                }
            }
        }
    }

    if ui.save_on_quit {
        Log::info(format!(
                "Writing updated markdown to file `{md_file}` and exiting..."
        ));
        let file = fs::File::create(md_file);
        if file.is_err() {
            Log::error(format!("Failed to open file for writing: `{md_file}`"));
            process::exit(ErrorCode::IO.into());
        }
        write!(file.unwrap(), "{}", ui.document.to_md()).unwrap_or_else(|e| {
            Log::error(format!("Failed to write to file: `{md_file}`: {e}"));
            process::exit(ErrorCode::IO.into());
        });
    }

    ui.deinit()?;
    Log::flush();
    Ok(())
}
