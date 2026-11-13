//! What rex prints. A repo decides the text of its own file names and doc
//! comments, and rex prints both, so the characters that would move a cursor
//! or split one entry over two lines are replaced first.

use std::borrow::Cow;

/// `text` with every control character replaced by a question mark.
///
/// A file name can hold a newline and a doc comment can hold an escape
/// sequence that clears the screen. Both reach a terminal, and an agent reads
/// one line of this output as one entry. `ls` prints a question mark for the
/// same characters.
pub(super) fn printable(text: &str) -> Cow<'_, str> {
    if !text.contains(char::is_control) {
        return Cow::Borrowed(text);
    }

    Cow::Owned(
        text.chars()
            .map(|character| {
                if character.is_control() {
                    '?'
                } else {
                    character
                }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_with_nothing_to_replace_is_not_copied() {
        assert!(matches!(printable("src/main.rs"), Cow::Borrowed(_)));
    }

    #[test]
    fn a_newline_a_tab_and_an_escape_each_become_a_question_mark() {
        assert_eq!(printable("two\nlines"), "two?lines");
        assert_eq!(printable("a\tb"), "a?b");
        assert_eq!(printable("clears \u{1b}[2J here"), "clears ?[2J here");
    }

    #[test]
    fn a_character_of_more_than_one_byte_is_kept_as_it_is() {
        assert_eq!(printable("café 😀"), "café 😀");
    }
}
