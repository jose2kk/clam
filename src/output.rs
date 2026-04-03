use owo_colors::OwoColorize;
use owo_colors::Stream::{Stderr, Stdout};

/// Print a success message to stdout in green (if TTY supports color).
pub fn success(msg: &str) {
    println!("{}", msg.if_supports_color(Stdout, |t| t.green()));
}

/// Print an error message to stderr in red (if TTY supports color).
pub fn error(msg: &str) {
    eprintln!("{}", msg.if_supports_color(Stderr, |t| t.red()));
}
