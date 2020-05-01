use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use clap::Clap;
use itertools::Itertools;
use regex::Regex;
use retain_mut::RetainMut;

use crate::srt::Subtitle;
use exitfailure::ExitFailure;
use failure::ResultExt;

mod srt;

fn clean_subtitle(subtitle: &mut Subtitle) {
    let regex = Regex::new(r"(?msU)♪.*♪|♪|#.*#|\(.*\)|^\s*[-‐]|[\p{Upper}\s]+:|<.*>").unwrap();

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

#[derive(Clap)]
struct Opts {
    /// The input file
    input: String,
}

fn main() -> Result<(), ExitFailure> {
    let opts: Opts = Opts::parse();

    let input_path = Path::new(&opts.input);
    let mut input_file = OpenOptions::new().read(true).write(true).create(false)
        .open(input_path).with_context(|_| format!("Could not open input {}", opts.input))?;
    let mut input_content = String::new();
    input_file.read_to_string(&mut input_content)
        .context("Could not read input")?;

    let old_path = input_path.with_extension("srt.old");
    let mut old_file = OpenOptions::new().write(true).create_new(true)
        .open(old_path).context("Could not open .old file")?;
    old_file.write_all(input_content.as_bytes())
        .context("Error while writing .old file")?;

    //ensure the input starts without whitespace and ends with two newlines
    //TODO find a way to do this without copying the string
    let mut input_content= input_content.trim().to_string();
    input_content += "\n\n";

    let mut subtitle = Subtitle::parse(&input_content)
        .context("Failed to parse subtitle")?;

    clean_subtitle(&mut subtitle);

    input_file.seek(SeekFrom::Start(0))
        .context("Error while seeking output")?;
    let new_content = subtitle.to_string();
    input_file.set_len(new_content.as_bytes().len() as u64)
        .context("Error while setting output file size")?;
    input_file.write_all(new_content.as_bytes())
        .context("Error while writing output")?;

    Ok(())
}
