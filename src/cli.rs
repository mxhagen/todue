use clap::{command, value_parser, Arg, ArgAction, Command, ValueHint};

pub fn new() -> Command {
    command!()
        .about("Manage todo lists through a tui/cli via markdown-files!")
        .arg(
            Arg::new("file")
                .help("Markdown file to use")
                .action(ArgAction::Set)
                .value_name("FILE")
                .value_parser(value_parser!(String))
                .value_hint(ValueHint::FilePath),
        )
}
