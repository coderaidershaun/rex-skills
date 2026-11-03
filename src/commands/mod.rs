//! One module per `rex` subcommand (`catalog` serves `rex commands`), each
//! exposing a `run` handler.

pub mod catalog;
pub mod codebase;
mod comments;
mod links;
mod rust_file;
pub mod skills;
pub mod smells;
mod stdout;
mod walk;
