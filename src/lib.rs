#![allow(clippy::arc_with_non_send_sync)]
pub use common::CliResult;

// near-tgi start
pub struct LogCollector {
    logs: Vec<String>,
}

// https://docs.rs/parse-ansi/0.1.6/src/parse_ansi/lib.rs.html#26
const ANSI_RE: &str = r"[\x1b\x9b]\[[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]";
lazy_static::lazy_static! {
    static ref ANSI_REGEX: regex::Regex = regex::Regex::new(ANSI_RE).unwrap();
}

pub fn escape_markdownv2(text: &str) -> String {
    const CHARS: [char; 19] = [
        '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
        '\\',
    ];

    text.chars()
        .fold(String::with_capacity(text.len()), |mut s, c| {
            if CHARS.contains(&c) {
                s.push('\\');
            }
            s.push(c);
            s
        })
}

pub fn escape_markdownv2_code(text: &str) -> String {
    const CHARS: [char; 2] = ['\\', '`'];

    text.chars()
        .fold(String::with_capacity(text.len()), |mut s, c| {
            if CHARS.contains(&c) {
                s.push('\\');
            }
            s.push(c);
            s
        })
}

pub fn escape_markdownv2_link(text: &str) -> String {
    const CHARS: [char; 3] = ['\\', ')', '`'];

    text.chars()
        .fold(String::with_capacity(text.len()), |mut s, c| {
            if CHARS.contains(&c) {
                s.push('\\');
            }
            s.push(c);
            s
        })
}

struct MarkdownEscape<T>(T);

impl<T> std::fmt::Display for MarkdownEscape<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "```\n{}\n```",
            escape_markdownv2_code(&format!("{}", self.0))
        )
    }
}

impl<T> std::fmt::Debug for MarkdownEscape<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "```json\n{:?}\n```",
            escape_markdownv2_code(&format!("{:?}", self.0))
        )
    }
}

impl LogCollector {
    fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn log(&mut self, log: String, escape_markdown: bool) {
        let log = ANSI_REGEX.replace_all(&log, "");

        let log: String = if escape_markdown {
            escape_markdownv2(&log)
        } else {
            log.to_string()
        };

        self.logs.push(log);
    }

    pub fn drain_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.logs)
    }
}

thread_local! {
    pub static LOG_COLLECTOR: std::cell::RefCell<LogCollector> = std::cell::RefCell::new(LogCollector::new());
}

/// Doesn't actually print to stderr, this macro saves all printed messages to be sent back to the user
#[macro_export]
macro_rules! eprintln {
    () => {
        $crate::LOG_COLLECTOR.with(|logger| {
            logger.borrow_mut().log("".to_string(), true);
        })
    };
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        $crate::LOG_COLLECTOR.with(|logger| {
            logger.borrow_mut().log(s, true);
        })
    }};
}

/// Doesn't actually print to stdout, this macro saves all printed messages to be sent back to the user
#[macro_export]
macro_rules! println {
    () => {
        $crate::LOG_COLLECTOR.with(|logger| {
            logger.borrow_mut().log("".to_string(), true);
        })
    };
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        $crate::LOG_COLLECTOR.with(|logger| {
            logger.borrow_mut().log(s, true);
        })
    }};
}

/// The string is already escaped
#[macro_export]
macro_rules! println_escaped {
    () => {
        $crate::LOG_COLLECTOR.with(|logger| {
            logger.borrow_mut().log("".to_string(), false);
        })
    };
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        $crate::LOG_COLLECTOR.with(|logger| {
            logger.borrow_mut().log(s, false);
        })
    }};
}

fn print_table(table: &prettytable::Table) {
    println_escaped!(
        "{}",
        table
            .row_iter()
            .map(
                |row| itertools::Itertools::chunks(row.iter().map(|s| s.get_content()), 3)
                    .into_iter()
                    .flat_map(|chunk| {
                        let mut chunk = chunk.into_iter();
                        let first = if let Some(first) = chunk.next() {
                            if !first.starts_with('`') && !first.is_empty() {
                                format!("*{first}*")
                            } else {
                                first.to_string()
                            }
                        } else {
                            "".to_string()
                        };
                        let second = if let Some(second) = chunk.next() {
                            if !second.starts_with('`') && !second.is_empty() {
                                format!("{second}")
                            } else {
                                second.to_string()
                            }
                        } else {
                            "".to_string()
                        };
                        let third = if let Some(third) = chunk.next() {
                            if !third.starts_with('`') && !third.is_empty() {
                                format!("_{third}_")
                            } else {
                                third.to_string()
                            }
                        } else {
                            "".to_string()
                        };
                        [first, second, third]
                    })
                    .filter(|s| !s.trim().is_empty())
                    .collect::<Vec<String>>()
                    .join(" : ")
                    + "\n"
            )
            .collect::<Vec<String>>()
            .join("\n")
    );
}
// near-tgi end

pub mod commands;
pub mod common;
pub mod config;
pub mod js_command_match;
pub mod network;
pub mod network_for_transaction;
pub mod network_view_at_block;
pub mod transaction_signature_options;
pub mod types;
pub mod utils_command;

#[derive(Debug, Clone)]
pub struct GlobalContext {
    pub config: crate::config::Config,
    pub offline: bool,
    pub teach_me: bool,
}
