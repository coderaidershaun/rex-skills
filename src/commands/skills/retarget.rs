//! Points the text of an installed skill at the folder it was installed into.
//! The bundle is written for Claude Code, so its text names `.claude/skills/`;
//! a copy for another tool must name that tool's folder instead.

use std::borrow::Cow;

use super::Vendor;

/// The folder the bundle text names.
const BUNDLE_SKILLS_DIR: &[u8] = b".claude/skills/";

impl Vendor {
    /// `contents` with each `.claude/skills/` replaced by this vendor's own
    /// skills folder. The text is returned as it is when nothing changes,
    /// which is always the case for Claude.
    pub(super) fn retarget(self, contents: &[u8]) -> Cow<'_, [u8]> {
        let own_dir = format!("{}/skills/", self.dir_name());
        if own_dir.as_bytes() == BUNDLE_SKILLS_DIR || find(contents, BUNDLE_SKILLS_DIR).is_none() {
            return Cow::Borrowed(contents);
        }

        let mut retargeted = Vec::with_capacity(contents.len());
        let mut rest = contents;
        while let Some(at) = find(rest, BUNDLE_SKILLS_DIR) {
            retargeted.extend_from_slice(&rest[..at]);
            retargeted.extend_from_slice(own_dir.as_bytes());
            rest = &rest[at + BUNDLE_SKILLS_DIR.len()..];
        }
        retargeted.extend_from_slice(rest);
        Cow::Owned(retargeted)
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_text_is_returned_as_it_is() {
        let text = b"Read .claude/skills/INDEX.md.";

        assert!(matches!(Vendor::Claude.retarget(text), Cow::Borrowed(_)));
    }

    #[test]
    fn every_mention_is_replaced_for_another_vendor() {
        let text = b"`.claude/skills/a/SKILL.md` and `.claude/skills/b/SKILL.md`";

        assert_eq!(
            Vendor::Codex.retarget(text).as_ref(),
            b"`.codex/skills/a/SKILL.md` and `.codex/skills/b/SKILL.md`"
        );
        assert_eq!(
            Vendor::Agents.retarget(text).as_ref(),
            b"`.agents/skills/a/SKILL.md` and `.agents/skills/b/SKILL.md`"
        );
    }

    #[test]
    fn text_without_the_folder_name_is_returned_as_it_is() {
        let text = b"Nothing to change here, not even .claude alone.";

        assert!(matches!(Vendor::Codex.retarget(text), Cow::Borrowed(_)));
    }
}
