use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use failure::{ensure, Error, ResultExt};
use itertools::Itertools;

use crate::clean::clean_subtitle;
use crate::srt::Subtitle;

mod clean;
mod srt;

#[cfg(test)]
mod test;

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();

    ensure!(args.len() == 2, "Usage: subclean [glob pattern]");
    let pattern = &args[1];

    // collect everything immediately so new files don't intervene
    let entries: Vec<_> = glob::glob(pattern)
        .with_context(|_| "Invalid glob pattern")?
        .into_iter()
        .try_collect()
        .with_context(|_| "Error during glob matching")?;

    for entry in entries {
        clean_single(entry)?;
    }

    Ok(())
}

fn clean_single(path: impl AsRef<Path>) -> Result<(), Error> {
    let input_path = path.as_ref().with_extension("srt");
    println!("Cleaning {:?}", input_path);

    let mut input_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open(&input_path)
        .with_context(|_| format!("Could not open input path {:?}", input_path))?;
    let mut input_content = String::new();
    input_file
        .read_to_string(&mut input_content)
        .context("Could not read input")?;

    let old_path = input_path.with_extension("srt.old");
    let mut old_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(old_path)
        .context("Could not open .old file")?;
    old_file
        .write_all(input_content.as_bytes())
        .context("Error while writing .old file")?;

    //ensure the input starts without whitespace and ends with two newlines
    //TODO find a way to do this without copying the string
    let mut input_content = input_content.trim().to_string();
    input_content += "\n\n";

    //TODO maybe move this to the parser
    let input_content = input_content.replace("\r\n", "\n");

    let mut subtitle = Subtitle::parse(&input_content).context("Failed to parse subtitle")?;

    clean_subtitle(&mut subtitle);

    input_file
        .seek(SeekFrom::Start(0))
        .context("Error while seeking output")?;
    let new_content = subtitle.to_string();
    input_file
        .set_len(new_content.as_bytes().len() as u64)
        .context("Error while setting output file size")?;
    input_file
        .write_all(new_content.as_bytes())
        .context("Error while writing output")?;

    Ok(())
}
