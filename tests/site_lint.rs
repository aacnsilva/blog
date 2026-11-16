//! Lints for the generated site and its Markdown sources.
//!
//! `site_contract.rs` pins what the site says; this file pins how it is built:
//! valid structure, working internal links, accessible markup, tidy sources.
//! Every rule reports all offenders at once so a failure is one fix-up pass.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Hand-written pages copied from `static/`. They get the universal rules
/// (links, ids, alt text, document basics) but not the generator's template rules.
const HAND_WRITTEN: &[&str] = &["justa/", "cv/", "joora/"];

/// Elements HTML has dropped, plus ones that never existed.
const OBSOLETE_ELEMENTS: &[&str] = &[
    "acronym", "big", "center", "content", "font", "marquee", "strike", "tt",
];

#[test]
fn generated_html_follows_the_rules() {
    let root = build_site("lint-html");
    let mut problems = Vec::new();

    for file in files_with_extension(&root, "html") {
        let relative = relative_path(&root, &file);
        let html = read(&file);
        let tags = tags(&html);
        let mut report = |message: String| problems.push(format!("{relative}: {message}"));

        check_document_basics(&html, &tags, &mut report);
        check_internal_links(&root, &tags, &mut report);
        check_unique_ids(&tags, &mut report);
        check_images_have_alt(&tags, &mut report);

        if !HAND_WRITTEN
            .iter()
            .any(|prefix| relative.starts_with(prefix))
        {
            check_template_rules(&relative, &tags, &mut report);
        }
    }

    assert_no_problems("generated HTML", &problems);
}

#[test]
fn markdown_sources_follow_the_rules() {
    let content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("content");
    let mut problems = Vec::new();

    for file in files_with_extension(&content, "md") {
        let relative = relative_path(&content, &file);
        let raw = read(&file);
        let mut report = |message: String| problems.push(format!("content/{relative}: {message}"));

        check_text_hygiene(&raw, &mut report);
        check_front_matter(&relative, &raw, &mut report);
    }

    assert_no_problems("Markdown sources", &problems);
}

fn check_document_basics(html: &str, tags: &[Tag], report: &mut impl FnMut(String)) {
    if !html
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("<!doctype html>")
    {
        report("missing <!DOCTYPE html>".to_string());
    }
    if !tags
        .iter()
        .any(|tag| tag.name == "html" && tag.attr("lang").is_some_and(|lang| !lang.is_empty()))
    {
        report("<html> needs a lang attribute".to_string());
    }
    if !tags
        .iter()
        .any(|tag| tag.name == "meta" && tag.attr("charset").is_some())
    {
        report("missing <meta charset>".to_string());
    }
    if !tags
        .iter()
        .any(|tag| tag.name == "meta" && tag.attr("name") == Some("viewport"))
    {
        report("missing viewport meta".to_string());
    }
    match text_between(html, "<title>", "</title>") {
        Some(title) if !title.trim().is_empty() => {}
        _ => report("missing or empty <title>".to_string()),
    }
}

fn check_internal_links(root: &Path, tags: &[Tag], report: &mut impl FnMut(String)) {
    for tag in tags {
        for attribute in ["href", "src"] {
            let Some(url) = tag.attr(attribute) else {
                continue;
            };
            if !url.starts_with('/') || url.starts_with("//") {
                continue;
            }
            let path = url.split(['#', '?']).next().unwrap_or_default();
            let target = root.join(path.trim_start_matches('/'));
            if !(target.is_file() || target.join("index.html").is_file()) {
                report(format!("<{}> points at missing {url}", tag.name));
            }
        }
    }
}

fn check_unique_ids(tags: &[Tag], report: &mut impl FnMut(String)) {
    let mut seen = BTreeSet::new();
    for id in tags.iter().filter_map(|tag| tag.attr("id")) {
        if !seen.insert(id) {
            report(format!("duplicate id \"{id}\""));
        }
    }
}

fn check_images_have_alt(tags: &[Tag], report: &mut impl FnMut(String)) {
    for tag in tags.iter().filter(|tag| tag.name == "img") {
        if tag.attr("alt").is_none() {
            report(format!(
                "<img src=\"{}\"> needs alt text (alt=\"\" if decorative)",
                tag.attr("src").unwrap_or_default()
            ));
        }
    }
}

fn check_template_rules(relative: &str, tags: &[Tag], report: &mut impl FnMut(String)) {
    for tag in tags {
        if OBSOLETE_ELEMENTS.contains(&tag.name.as_str()) {
            report(format!("<{}> is not a valid HTML element", tag.name));
        }
    }

    let headings: Vec<usize> = tags.iter().filter_map(Tag::heading_level).collect();
    let h1_count = headings.iter().filter(|level| **level == 1).count();
    if h1_count != 1 {
        report(format!("expected exactly one <h1>, found {h1_count}"));
    }
    // The site title in the header is an <h2> today, so start checking at the <h1>.
    let from_h1 = headings.iter().skip_while(|level| **level != 1);
    let mut previous = 1;
    for level in from_h1 {
        if *level > previous + 1 {
            report(format!("heading jumps from <h{previous}> to <h{level}>"));
        }
        previous = *level;
    }

    let has_description = tags.iter().any(|tag| {
        tag.name == "meta"
            && tag.attr("name") == Some("description")
            && tag
                .attr("content")
                .is_some_and(|content| !content.trim().is_empty())
    });
    if !has_description {
        report("missing or empty meta description".to_string());
    }

    let has_canonical = tags.iter().any(|tag| {
        tag.name == "link"
            && tag.attr("rel") == Some("canonical")
            && tag
                .attr("href")
                .is_some_and(|href| href.starts_with("https://"))
    });
    if relative != "404.html" && !has_canonical {
        report("missing absolute canonical link".to_string());
    }
}

fn check_text_hygiene(raw: &str, report: &mut impl FnMut(String)) {
    if !raw.ends_with('\n') || raw.ends_with("\n\n") {
        report("file must end with exactly one newline".to_string());
    }
    if raw.contains('\r') {
        report("use LF line endings".to_string());
    }
    let mut in_code_block = false;
    for (index, line) in raw.lines().enumerate() {
        let number = index + 1;
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
        }
        // Two trailing spaces are a Markdown hard break; anything else is noise.
        let trailing = line.len() - line.trim_end().len();
        if trailing > 0 && (in_code_block || trailing != 2 || line.trim().is_empty()) {
            report(format!("line {number}: trailing whitespace"));
        }
        if !in_code_block && line.contains('\t') {
            report(format!("line {number}: tab character outside a code block"));
        }
    }
    if in_code_block {
        report("unclosed ``` code block".to_string());
    }
}

fn check_front_matter(relative: &str, raw: &str, report: &mut impl FnMut(String)) {
    let Some(front_matter) = raw
        .strip_prefix("+++\n")
        .and_then(|rest| rest.split_once("\n+++\n"))
        .map(|(front_matter, _)| front_matter)
    else {
        report("must start with a +++ TOML front matter block".to_string());
        return;
    };
    let value = |key: &str| {
        front_matter.lines().find_map(|line| {
            let (name, value) = line.split_once('=')?;
            (name.trim() == key).then(|| value.trim().trim_matches(['\'', '"']).to_string())
        })
    };

    if value("title").is_none_or(|title| title.is_empty()) {
        report("front matter needs a title".to_string());
    }

    let is_post = relative.starts_with("blog/") && !relative.ends_with("_index.md");
    if !is_post {
        return;
    }
    match value("date") {
        Some(date) if is_rfc3339(&date) => {}
        Some(date) => report(format!(
            "date \"{date}\" must look like 2026-01-31T09:30:00+00:00"
        )),
        None => report("posts need a date".to_string()),
    }
    if !matches!(value("draft").as_deref(), Some("true" | "false")) {
        report("posts need an explicit draft = true|false".to_string());
    }
}

fn is_rfc3339(date: &str) -> bool {
    const SHAPE: &str = "dddd-dd-ddTdd:dd:dd";
    let Some((stamp, offset)) = date.split_at_checked(SHAPE.len()) else {
        return false;
    };
    let matches_shape = |text: &str, shape: &str| {
        text.len() == shape.len()
            && text.chars().zip(shape.chars()).all(|(actual, expected)| {
                if expected == 'd' {
                    actual.is_ascii_digit()
                } else {
                    actual == expected
                }
            })
    };
    matches_shape(stamp, SHAPE)
        && (offset == "Z" || matches_shape(offset, "+dd:dd") || matches_shape(offset, "-dd:dd"))
}

/// An opening tag: lowercase name plus its attributes.
struct Tag {
    name: String,
    attributes: Vec<(String, String)>,
}

impl Tag {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    fn heading_level(&self) -> Option<usize> {
        match self.name.as_bytes() {
            [b'h', level @ b'1'..=b'6'] => Some(usize::from(level - b'0')),
            _ => None,
        }
    }
}

/// Collects opening tags, skipping comments and the bodies of scripts and styles.
/// Small on purpose: it reads the HTML this site emits, not HTML in general.
fn tags(html: &str) -> Vec<Tag> {
    let mut tags = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        rest = &rest[start..];
        if let Some(after) = rest.strip_prefix("<!--") {
            rest = after.split_once("-->").map_or("", |(_, tail)| tail);
            continue;
        }
        let Some(end) = tag_end(rest) else { break };
        let inner = &rest[1..end];
        rest = &rest[end + 1..];
        if !inner.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
            continue;
        }
        let tag = parse_tag(inner);
        if tag.name == "script" || tag.name == "style" {
            let closing = format!("</{}", tag.name);
            rest = rest.find(&closing).map_or("", |index| &rest[index..]);
        }
        tags.push(tag);
    }
    tags
}

/// Index of the `>` that closes the tag starting at `text[0]`, ignoring `>` inside quotes.
fn tag_end(text: &str) -> Option<usize> {
    let mut quote = None;
    for (index, ch) in text.char_indices() {
        match (quote, ch) {
            (None, '>') => return Some(index),
            (None, '"' | '\'') => quote = Some(ch),
            (Some(open), _) if open == ch => quote = None,
            _ => {}
        }
    }
    None
}

fn parse_tag(inner: &str) -> Tag {
    let inner = inner.trim_end_matches('/');
    let name_end = inner
        .find(|ch: char| ch.is_whitespace())
        .unwrap_or(inner.len());
    let mut attributes = Vec::new();
    let mut rest = inner[name_end..].trim_start();
    while !rest.is_empty() {
        let key_end = rest
            .find(|ch: char| ch == '=' || ch.is_whitespace())
            .unwrap_or(rest.len());
        let key = rest[..key_end].to_ascii_lowercase();
        rest = rest[key_end..].trim_start();
        let mut value = String::new();
        if let Some(after) = rest.strip_prefix('=') {
            let after = after.trim_start();
            let (raw, tail) = match after.chars().next() {
                Some(quote @ ('"' | '\'')) => {
                    after[1..].split_once(quote).unwrap_or((&after[1..], ""))
                }
                _ => after.split_once(char::is_whitespace).unwrap_or((after, "")),
            };
            value = raw.to_string();
            rest = tail.trim_start();
        }
        if !key.is_empty() {
            attributes.push((key, value));
        }
    }
    Tag {
        name: inner[..name_end].to_ascii_lowercase(),
        attributes,
    }
}

fn text_between<'a>(text: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let (_, after) = text.split_once(open)?;
    after.split_once(close).map(|(inner, _)| inner)
}

fn assert_no_problems(subject: &str, problems: &[String]) {
    assert!(
        problems.is_empty(),
        "{} problem(s) in {subject}:\n  {}\n",
        problems.len(),
        problems.join("\n  ")
    );
}

fn build_site(name: &str) -> PathBuf {
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(name);
    if output.exists() {
        fs::remove_dir_all(&output).expect("stale lint output is removable");
    }
    let status = Command::new(env!("CARGO_BIN_EXE_aacnsilva-blog"))
        .arg("--output")
        .arg(&output)
        .status()
        .expect("the site generator runs");
    assert!(
        status.success(),
        "the site generator should exit successfully"
    );
    output
}

fn files_with_extension(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(current) = pending.pop() {
        let entries = fs::read_dir(&current)
            .unwrap_or_else(|error| panic!("failed to list {}: {error}", current.display()));
        for entry in entries {
            let path = entry.expect("directory entries are readable").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|found| found == extension) {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn relative_path(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .expect("file is under root")
        .to_string_lossy()
        .replace('\\', "/")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
