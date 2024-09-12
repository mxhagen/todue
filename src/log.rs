use std::{
    io::{self, Write},
    sync::Mutex,
};

pub struct Log {}
pub struct DropGuard<F>
where
    F: Fn(),
{
    pub exec_on_drop: F,
}

static LOG_BUFFER: Mutex<String> = Mutex::new(String::new());

impl Log {
    pub fn log<S1, S2, S3>(label: S1, message: S2, label_ansi_style_code: S3)
    where
        S1: ToString,
        S2: ToString,
        S3: ToString,
    {
        let mut log_entry = String::new();
        log_entry += &label_ansi_style_code.to_string(); // set style
        log_entry += &label.to_string();
        log_entry += "\x1b[0m"; // reset style
        log_entry += &message.to_string();
        log_entry += "\r\n";

        *LOG_BUFFER.lock().unwrap() += &log_entry;
    }

    pub fn error_exit_with<S, I>(error_code: I, error_msg: S)
    where
        S: ToString,
        I: Into<i32>,
    {
        Self::error(error_msg);
        Self::flush();
        std::process::exit(error_code.into());
    }

    pub fn error<S>(error_msg: S)
    where
        S: ToString,
    {
        Self::log("error:   ", error_msg, "\x1b[1;31m"); // red and bold label style
    }

    pub fn info<S>(info_msg: S)
    where
        S: ToString,
    {
        Self::log("info:    ", info_msg, "\x1b[1;34m"); // blue and bold label style
    }

    pub fn warn<S>(warning_msg: S)
    where
        S: ToString,
    {
        Self::log("warning: ", warning_msg, "\x1b[1;33m"); // blue and bold label style
    }

    pub fn flush() {
        let _ = write!(&mut io::stderr(), "{}", *LOG_BUFFER.lock().unwrap());
        io::stderr().flush().unwrap();
    }
}

// unused variants here are ok -> hide warning in debug builds
#[cfg_attr(debug_assertions, allow(dead_code))]
pub enum ErrorCode {
    App = 1,
    IO = 2,
}

impl From<ErrorCode> for i32 {
    fn from(value: ErrorCode) -> Self {
        value as Self
    }
}

impl<F> Drop for DropGuard<F>
where
    F: Fn(),
{
    fn drop(&mut self) {
        (self.exec_on_drop)()
    }
}
