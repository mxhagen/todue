use anyhow::anyhow;
use chrono::NaiveDateTime;

/// a todo-list entry; can be thought of as an abstract representation of a line
/// of markdown in one of the following formats:
/// - without deadline: `"- [ ] Do something"`
/// - with deadline: `"- [ ] (2024-06-20 20:00) Do another thing"`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Entry {
    pub done: bool,
    pub text: String,
    pub deadline: Option<NaiveDateTime>,
}

/// a todo-document; can be thought of as an abstract representation of an entire
/// document consisting of `title`, and all of the documents `entries`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Document {
    pub title: Option<String>,
    pub entries: Vec<Entry>,
}

pub trait Markdown {
    fn to_md(&self) -> String;
    fn from_md(md: String) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl Markdown for Entry {
    fn to_md(&self) -> String {
        let mut md = format!("- [{}] ", if self.done { "x" } else { " " });
        if let Some(deadline) = self.deadline {
            md += &format!("({}) ", deadline.format("(%Y-%m-%d %H:%M)"));
        }
        md += &self.text;
        md
    }

    fn from_md(md: String) -> anyhow::Result<Self> {
        let mut entry = Entry::default();

        #[derive(PartialEq, Eq)]
        enum ParseState {
            Nothing,
            Dash,
            BrackOpen,
            BrackClose,
            Text,
        }
        use ParseState::*;

        let mut state = Nothing;
        let mut i = 0;
        let mut text = String::new();

        while i < md.len() {
            match state {
                Nothing if &md[i..=i] == "-" => state = Dash,

                Dash if &md[i..=i] == "[" => state = BrackOpen,

                BrackOpen if &md[i..=i] == "x" => entry.done = true,

                BrackOpen if &md[i..=i] == "]" => state = BrackClose,

                BrackOpen
                    if !&md[i..=i]
                        .chars()
                        .next()
                        .unwrap_or(' ')
                        .is_ascii_whitespace() =>
                {
                    return Err(anyhow!("Invalid character in presumed checkbox. Expected whitespace or 'x', found '{}'", &md[i..=i]));
                }

                BrackClose if md.len() - i >= "(YYYY-mm-dd HH:MM)".len() => {
                    let maybe_date = &md[i..i + "(YYYY-mm-dd HH:MM)".len()];
                    let deadline = NaiveDateTime::parse_from_str(maybe_date, "(%Y-%m-%d %H:%M)");
                    if maybe_date.chars().any(|c| c.is_alphabetic()) {
                        state = Text; // if there is any letters, stop trying to parse date
                    } else {
                        match deadline {
                            Ok(deadline) => {
                                entry.deadline = Some(deadline);
                                state = Text;
                                i += "(YYYY-mm-dd HH:MM)".len();
                                text = String::new();
                            }
                            _ => text += &md[i..=i],
                        }
                    }
                }

                Text | BrackClose
                    if !&md[i..].chars().next().unwrap_or(' ').is_ascii_whitespace() =>
                {
                    text += &md[i..=i];
                    state = Text;
                }

                Text if !["\r", "\n"].contains(&&md[i..=i]) => text += &md[i..=i],

                _ => {}
            }
            i += 1;
        }

        match state {
            Text => {
                entry.text = text.trim_start().to_string();
                Ok(entry)
            }
            _ => Err(anyhow!("Invalid entry")),
        }
    }
}

impl Markdown for Document {
    fn to_md(&self) -> String {
        let mut md = match &self.title {
            None => String::new(),
            Some(title) => format!("# {}\n", title),
        };

        if !self.entries.is_empty() {
            md += "\n";
            for entry in &self.entries {
                md += &(entry.to_md() + "\n");
            }
        }

        md
    }

    fn from_md(md: String) -> anyhow::Result<Self> {
        let mut document = Document::default();
        for line in md.lines() {
            if document.title.is_none() {
                let mut chars = line.chars().peekable();
                let mut count = 0;
                while chars.peek().is_some_and(|c| c.is_ascii_whitespace()) {
                    chars.next();
                };
                while chars.peek().is_some_and(|&c| c == '#') {
                    count += 1;
                    chars.next();
                };
                while chars.peek().is_some_and(|c| c.is_ascii_whitespace()) {
                    chars.next();
                };
                if count == 1 {
                    document.title = Some(chars.collect());
                }
            }
            if let Ok(entry) = Entry::from_md(line.to_string()) {
                document.entries.push(entry);
            }
        }
        Ok(document)
    }
}

