#![warn(clippy::disallowed_methods)]

#[macro_use]
extern crate shadow_rs;

shadow!(shadow);

// Lib is present to allow for benchmarking
pub mod bug_report;
pub mod config;
pub mod configs;
pub mod configure;
pub mod context;
pub mod context_env;
pub mod formatter;
pub mod init;
pub mod logger;
pub mod module;
mod modules;
pub mod print;
mod segment;
mod serde_utils;
mod utils;
#[cfg(target_os = "windows")]
pub mod win_fast_spawn;

#[cfg(test)]
mod test;
