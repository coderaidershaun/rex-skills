//! Implementation behind the `rex` binary. Command logic lives here in
//! library form so tests can call it directly instead of spawning the CLI.

pub mod bundle;
pub mod commands;
pub mod error;
