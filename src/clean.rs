use itertools::Itertools;
use regex::Regex;
use retain_mut::RetainMut;

use crate::srt::Subtitle;

const PATTERNS: &[&'static str] = &[
    r"♪.*♪",
    r"♪",
    r"#.*#",
    r"\(.*\)",
    r"\[.*\]",
    r"^\s*[-‐]",
    r"\p{Upper}[\p{Upper}\s\d]*:",
    r"<.*>",
];

pub fn clean_subtitle(subtitle: &mut Subtitle) {
    let pattern = "(?msU)".to_string() + &PATTERNS.iter().join("|");
    let regex = Regex::new(&pattern).unwrap();

    subtitle.blocks.retain_mut(|block| {
        let replaced = regex.replace_all(&block.text, "");
        let stripped = replaced.lines().map(str::trim).filter(|s| !s.is_empty()).join("\n");

        if stripped.is_empty() {
            false
        } else {
            block.text = stripped;
            true
        }
    });
}

