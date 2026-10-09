//! Lists every `SMELL` flag comment in the Rust files of the working
//! directory. A flag marks a known problem that was left in the code, and this
//! command is how an agent or a person finds them all again.

use std::fs;
use std::path::Path;

use super::stdout::print_to_stdout;
use super::walk::visible_entries;
use crate::error::RexError;

const FLAG_WORD: &[u8] = b"SMELL";

/// Walk `cwd`, skip dotfiles and gitignored paths, and print one `path:line`
/// for each `SMELL` flag comment in a Rust file, then the count of them all.
///
/// # Errors
/// - [`RexError::Walk`] if the directory walk fails (loop, glob, permissions).
/// - [`RexError::Io`] if a Rust file cannot be read.
/// - [`RexError::Stdout`] if the report cannot be written to stdout.
pub fn run(cwd: &Path) -> Result<(), RexError> {
    print_to_stdout(&render_report(cwd)?)
}

fn render_report(cwd: &Path) -> Result<String, RexError> {
    let mut out = String::new();
    let mut flag_count = 0;

    for entry in visible_entries(cwd) {
        let entry = entry?;
        let path = entry.path();

        let is_rust_file = entry.file_type().is_some_and(|ft| ft.is_file())
            && path.extension().is_some_and(|ext| ext == "rs");
        if !is_rust_file {
            continue;
        }

        // The file is read as bytes because text that is not valid UTF-8 must
        // not fail the whole command.
        let source = fs::read(path).map_err(|source| RexError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let shown_path = path.strip_prefix(cwd).unwrap_or(path).display();

        for (index, line) in source.split(|&byte| byte == b'\n').enumerate() {
            if has_flag(line) {
                out.push_str(&format!("{shown_path}:{}\n", index + 1));
                flag_count += 1;
            }
        }
    }

    out.push_str(&format!("Code smells: {flag_count}\n"));
    Ok(out)
}

/// Tells if a line holds a `//` comment whose text starts with the flag word.
///
/// A comment that only talks about the word further in is not a flag, and
/// neither is a longer word that starts the same way, such as `SMELLY`.
fn has_flag(line: &[u8]) -> bool {
    let Some(comment) = line_comment(line) else {
        return false;
    };

    // Doc comments open with `///` or `//!`, so those marks are skipped along
    // with the spaces.
    let text_start = comment
        .iter()
        .position(|byte| !matches!(byte, b'/' | b'!') && !byte.is_ascii_whitespace())
        .unwrap_or(comment.len());

    comment[text_start..]
        .strip_prefix(FLAG_WORD)
        .is_some_and(|rest| {
            rest.first()
                .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
        })
}

/// Finds the text of the `//` comment on a line, if the line has one. A `//`
/// inside a string, such as a web address, does not start a comment.
///
/// Known limit: each line is read on its own. A line inside a string or a
/// `/* */` comment that began on a line above is read as code, and a `"`
/// written as a single character (`'"'`) is read as the start of a string.
fn line_comment(line: &[u8]) -> Option<&[u8]> {
    let mut is_in_string = false;
    let mut index = 0;

    while index < line.len() {
        match line[index] {
            // The character after a backslash is part of the string, even
            // when it is a quote.
            b'\\' if is_in_string => index += 1,
            b'"' => is_in_string = !is_in_string,
            b'/' if !is_in_string && line.get(index + 1) == Some(&b'/') => {
                return Some(&line[index + 2..]);
            }
            _ => {}
        }
        index += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_flag_accepts_a_comment_that_starts_with_the_flag_word() {
        let flagged = [
            "// SMELL: bad name",
            "    // SMELL: indented",
            "//SMELL: no space",
            "// SMELL",
            "// SMELL(naming): tagged",
            "/// SMELL: on a doc comment",
            "//! SMELL: on a module doc comment",
            "let total = 1; // SMELL: magic number",
            "let url = \"http://example.com\"; // SMELL: fixed address",
            "let quote = \"say \\\"hi\\\"\"; // SMELL: escaped quotes before it",
            "// SMELL: windows line ending\r",
        ];

        for line in flagged {
            assert!(has_flag(line.as_bytes()), "expected a flag in: {line}");
        }
    }

    #[test]
    fn has_flag_rejects_a_line_with_no_flag_comment() {
        let clean = [
            "",
            "fn main() {}",
            "// a plain comment",
            "// This removes the SMELL: flag from the list.",
            "// SMELLY code lives here",
            "// SMELL_COUNT is the total",
            "// smell: lower case",
            "const FLAG: &str = \"SMELL:\";",
            "let fixture = \"// SMELL: inside a string\";",
        ];

        for line in clean {
            assert!(!has_flag(line.as_bytes()), "expected no flag in: {line}");
        }
    }
}
