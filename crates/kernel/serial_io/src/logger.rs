use crate::access_serial;
use core::{fmt, fmt::Write};
use log::{Metadata, Record};

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        access_serial(|serial| {
            DecoratedLog::write(serial, record.level(), record.args()).unwrap()
        })
    }

    fn flush(&self) {}
}

pub static LOGGER: Logger = Logger;

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
}

/// Stolen directly from the uefi crate, because they have a great implementation
///
/// Writer wrapper which prints a log level in front of every line of text
///
/// This is less easy than it sounds because...
///
/// 1. The fmt::Arguments is a rather opaque type, the ~only thing you can do
///    with it is to hand it to an fmt::Write implementation.
/// 2. Without using memory allocation, the easy cop-out of writing everything
///    to a String then post-processing is not available.
///
/// Therefore, we need to inject ourselves in the middle of the fmt::Write
/// machinery and intercept the strings that it sends to the Writer.
struct DecoratedLog<'writer, W: fmt::Write> {
    writer: &'writer mut W,
    log_level: log::Level,
    at_line_start: bool,
}

impl<'writer, W: fmt::Write> DecoratedLog<'writer, W> {
    // Call this method to print a level-annotated log
    fn write(
        writer: &'writer mut W,
        log_level: log::Level,
        args: &fmt::Arguments,
    ) -> fmt::Result {
        let mut decorated_writer = Self {
            writer,
            log_level,
            at_line_start: true,
        };
        writeln!(decorated_writer, "{}", *args)
    }
}

impl<'writer, W: fmt::Write> fmt::Write for DecoratedLog<'writer, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Split the input string into lines
        let mut lines = s.lines();

        // The beginning of the input string may actually fall in the middle of
        // a line of output. We only print the log level if it truly is at the
        // beginning of a line of output.
        let first = lines.next().unwrap_or("");
        if self.at_line_start {
            write!(self.writer, "{}: ", self.log_level)?;
            self.at_line_start = false;
        }
        write!(self.writer, "{}", first)?;

        // For the remainder of the line iterator (if any), we know that we are
        // truly at the beginning of lines of output.
        for line in lines {
            write!(self.writer, "\n{}: {}", self.log_level, line)?;
        }

        // If the string ends with a newline character, we must 1/propagate it
        // to the output (it was swallowed by the iteration) and 2/prepare to
        // write the log level of the beginning of the next line (if any).
        if let Some('\n') = s.chars().next_back() {
            writeln!(self.writer)?;
            self.at_line_start = true;
        }
        Ok(())
    }
}
