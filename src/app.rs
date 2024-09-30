use crate::*;

use std::io::Stdout;

#[derive()]
pub struct App {
    pub ui: Ui<Stdout>,
    pub md_file: String,
    pub running: bool,
    pub keymap: Keymap,
    pub mode: Mode,
}

impl App {
    pub fn init() -> anyhow::Result<Self> {
        let cli = cli::new();
        let args = cli.clone().get_matches();

        let md_file = match args.get_one::<String>("file") {
            Some(path) => path.into(),
            _ => {
                Log::warn("No filename specified -- falling back to `todo.md`");
                "todo.md".to_string()
            }
        };

        if !path::Path::new(&md_file).exists() {
            Log::error_exit_with(
                ErrorCode::IO,
                format!("Markdown file `{md_file}` not found. Exiting..."),
            );
        }

        let md = fs::read_to_string(&md_file).unwrap();
        let document = Document::from_md(md)?;

        let ui = Ui::init(io::stdout(), document);

        Ok(Self {
            ui,
            md_file,
            running: true,
            keymap: Keymap::default(),
            mode: Mode::Normal,
        })
    }

    pub fn handle_input(&mut self) -> anyhow::Result<()> {
        if poll(time::Duration::from_millis(250)).unwrap_or(false) {
            if let Ok(Key(k)) = read() {
                let a = self.keymap.map.get(&(self.mode.clone(), k));
                if let Some(callback) = a {
                    (callback.clone())(self);
                }
            }
        }
        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
