use std::borrow::Cow;

use itertools::Itertools;
use regex::Regex;

use crate::srt::Subtitle;

const PATTERNS: &[&str] = &[
    r"♪.*♪",
    r"♪",
    r"#.*#",
    r"\(.*\)",
    r"\[.*\]",
    r"^\s*[-‐]",
    r"-?\p{Upper}[\w\s\d\-_]*:",
    r"<.*>",
    r"Subtitles downloaded from www\.OpenSubtitles\.org",
];

fn remove_regex_repeated<'s>(regex: &Regex, text: &'s str) -> Cow<'s, str> {
    let mut result = Cow::Borrowed(text);
    loop {
        let new = regex.replace_all(&result, "");
        match new {
            Cow::Borrowed(_) => return result,
            Cow::Owned(new) => result = Cow::Owned(new),
        }
    }
}

pub fn clean_subtitle(subtitle: &mut Subtitle) {
    let pattern = "(?msU)".to_string() + &PATTERNS.iter().join("|");
    let regex = Regex::new(&pattern).unwrap();

    subtitle.blocks.retain_mut(|block| {
        let replaced = remove_regex_repeated(&regex, &block.text);
        let stripped = replaced
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .join("\n");

        if stripped.is_empty() {
            false
        } else {
            block.text = stripped;
            true
        }
    })
}
