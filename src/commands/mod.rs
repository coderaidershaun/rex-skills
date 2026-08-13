//! One module per `rex` subcommand, each exposing a `run` handler.

pub mod codebase;
#[allow(clippy::module_inception)]
pub mod commands;
pub mod skills;
