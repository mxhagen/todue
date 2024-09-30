pub use std::{
    fs,
    io::{self, Write},
    path, process, time,
};

use crossterm::event::{poll, read, Event::*, KeyCode::Char};

mod app;
use app::*;

mod control;
use control::*;

mod md;
use md::*;

mod ui;
use ui::*;

mod cli;

mod log;
pub use log::*;

mod tests;

fn main() -> anyhow::Result<()> {
    let mut app = App::init()?;

    let _guard = DropGuard {
        // clean up terminal state even on panics
        exec_on_drop: || {
            // needs two braces to function properly
            let _ = Ui::init(&mut io::stdout(), Document::default()).deinit();
            Log::flush();
        },
    };

    while app.running {
        app.ui.draw()?;
        app.handle_input()?;
    }

    if app.ui.save_on_quit {
        Log::info(format!(
            "Writing updated markdown to file `{}` and exiting...",
            app.md_file
        ));
        let file = fs::File::create(&app.md_file);
        if file.is_err() {
            Log::error(format!(
                "Failed to open file for writing: `{}`",
                app.md_file
            ));
            process::exit(ErrorCode::IO.into());
        }
        write!(file.unwrap(), "{}", app.ui.document.to_md()).unwrap_or_else(|e| {
            Log::error(format!("Failed to write to file: `{}`: {}", app.md_file, e));
            process::exit(ErrorCode::IO.into());
        });
    }

    app.ui.deinit()?;
    Log::flush();
    Ok(())
}
