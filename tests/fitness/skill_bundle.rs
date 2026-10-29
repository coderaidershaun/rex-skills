use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use yaml_rust2::YamlLoader;

const SKILLS_DIR: &str = ".claude/skills/";

/// Every embedded file with its text. The bundle is Markdown, so a file that
/// is not UTF-8 is a mistake.
fn bundle_files() -> Vec<(PathBuf, String)> {
    rex_skills::bundle::walk()
        .into_iter()
        .map(|(path, bytes)| {
            let text = String::from_utf8(bytes.to_vec())
                .unwrap_or_else(|_| panic!("{} is not UTF-8", path.display()));
            (path, text)
        })
        .collect()
}

/// The folder name and text of each `skills/<name>/SKILL.md`.
fn skills() -> Vec<(String, String)> {
    bundle_files()
        .into_iter()
        .filter_map(|(path, text)| {
            let parts: Vec<_> = path.iter().map(|part| part.to_string_lossy()).collect();
            match parts.as_slice() {
                [skills, folder, file] if skills == "skills" && file == "SKILL.md" => {
                    Some((folder.to_string(), text))
                }
                _ => None,
            }
        })
        .collect()
}

fn frontmatter(folder: &str, text: &str) -> String {
    let inner = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.find("\n---\n").map(|end| &rest[..end]));
    inner
        .unwrap_or_else(|| panic!("{folder}: SKILL.md must open with a --- frontmatter block"))
        .to_string()
}

fn parse(folder: &str, frontmatter: &str) -> yaml_rust2::Yaml {
    YamlLoader::load_from_str(frontmatter)
        .unwrap_or_else(|error| panic!("{folder}: frontmatter is not valid YAML: {error}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{folder}: frontmatter is empty"))
}

// This test must never be deleted. A skill whose name does not match its
// folder, or whose description is missing, is never found by the tool that
// reads the folder, and nothing reports it.
#[test]
fn every_skill_has_a_name_that_is_its_folder_and_a_description() {
    let skills = skills();
    assert!(!skills.is_empty(), "the bundle must hold skills");

    for (folder, text) in skills {
        let doc = parse(&folder, &frontmatter(&folder, &text));

        assert_eq!(
            doc["name"].as_str(),
            Some(folder.as_str()),
            "{folder}: name must be the folder name"
        );
        assert!(
            doc["description"]
                .as_str()
                .is_some_and(|description| !description.trim().is_empty()),
            "{folder}: description must not be empty"
        );
    }
}

// This test must never be deleted. A space followed by `#` starts a comment in
// a plain YAML value, so `Inline #[cfg(test)]` cuts a description down to its
// first words, and the skill stops loading when a person asks for it.
#[test]
fn a_plain_description_is_not_shortened_by_yaml() {
    for (folder, text) in skills() {
        let frontmatter = frontmatter(&folder, &text);
        let Some(written) = frontmatter
            .lines()
            .find_map(|line| line.strip_prefix("description:"))
            .map(str::trim)
        else {
            continue;
        };
        if written.is_empty() || written.starts_with(['>', '|', '\'', '"']) {
            continue;
        }

        let doc = parse(&folder, &frontmatter);
        assert_eq!(
            doc["description"].as_str(),
            Some(written),
            "{folder}: YAML changed the description. Quote it, because ` #` starts a comment"
        );
    }
}

/// The skill folder named by each `.claude/skills/<name>/SKILL.md` in `text`.
fn skills_in_paths(text: &str) -> BTreeSet<String> {
    text.match_indices(SKILLS_DIR)
        .filter_map(|(at, _)| {
            let after = &text[at + SKILLS_DIR.len()..];
            let name: String = after
                .chars()
                .take_while(|c| c.is_ascii_lowercase() || *c == '-')
                .collect();
            after[name.len()..].starts_with("/SKILL.md").then_some(name)
        })
        .collect()
}

#[test]
fn the_index_lists_every_skill_and_only_skills_that_exist() {
    let existing: BTreeSet<String> = skills().into_iter().map(|(folder, _)| folder).collect();
    let index = bundle_files()
        .into_iter()
        .find(|(path, _)| path == Path::new("skills/INDEX.md"))
        .expect("the bundle must hold skills/INDEX.md")
        .1;

    assert_eq!(
        skills_in_paths(&index),
        existing,
        "INDEX.md must list the path of each skill, and no other"
    );
}

/// Each `rex-…` word in `text`: lowercase letters joined by single hyphens.
fn rex_names(text: &str) -> Vec<&str> {
    text.match_indices("rex-")
        .filter(|(at, _)| {
            !text[..*at].ends_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
        .map(|(at, _)| {
            let bytes = &text.as_bytes()[at..];
            let mut end = 0;
            while let Some(&byte) = bytes.get(end) {
                let joins_a_letter =
                    byte == b'-' && bytes.get(end + 1).is_some_and(u8::is_ascii_lowercase);
                if !byte.is_ascii_lowercase() && !joins_a_letter {
                    break;
                }
                end += 1;
            }
            &text[at..at + end]
        })
        .collect()
}

#[test]
fn every_skill_name_used_in_the_bundle_is_a_skill_that_exists() {
    let existing: BTreeSet<String> = skills().into_iter().map(|(folder, _)| folder).collect();

    for (path, text) in bundle_files() {
        for name in rex_names(&text) {
            assert!(
                existing.contains(name),
                "{} names the skill `{name}`, which is not in the bundle",
                path.display()
            );
        }
    }
}

/// The path a Markdown link points to, relative to the bundle root, when it
/// is a link to another Markdown file.
fn markdown_link_target(from: &Path, link: &str) -> Option<PathBuf> {
    let file = link.split('#').next()?;
    if !file.ends_with(".md") || file.contains("://") {
        return None;
    }

    let mut target = from.parent()?.to_path_buf();
    for component in Path::new(file).components() {
        match component {
            Component::ParentDir => {
                target.pop();
            }
            Component::Normal(part) => target.push(part),
            _ => {}
        }
    }
    Some(target)
}

#[test]
fn every_markdown_link_to_a_file_in_the_bundle_resolves() {
    let files = bundle_files();
    let existing: BTreeSet<&Path> = files.iter().map(|(path, _)| path.as_path()).collect();

    for (path, text) in &files {
        for (at, _) in text.match_indices("](") {
            let rest = &text[at + 2..];
            let Some(link) = rest.find(')').map(|end| &rest[..end]) else {
                continue;
            };
            let Some(target) = markdown_link_target(path, link) else {
                continue;
            };
            assert!(
                existing.contains(target.as_path()),
                "{} links to `{link}`, which is not in the bundle",
                path.display()
            );
        }
    }
}
