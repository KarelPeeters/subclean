use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use failure::{Error, ResultExt};
use itertools::Itertools;

use crate::clean::clean_subtitle;
use crate::srt::Subtitle;

mod clean;
mod srt;

#[cfg(test)]
mod test;

const USAGE: &str = "Usage: subclean [path] or subclean glob [glob pattern]";

fn main() -> Result<(), Error> {
    let mut args: Vec<String> = std::env::args().collect();
    let is_dry = |a: &str| a == "-d" || a == "--dry";
    let dry = args.iter().any(|a| is_dry(a));
    args = args.into_iter().filter(|a| !is_dry(a)).collect_vec();

    let (glob, pattern) = if args.len() == 2 {
        if args[1] == "--help" || args[1] == "-h" {
            println!("{}", USAGE);
            return Ok(());
        }

        (false, &args[1])
    } else {
        if args.len() != 3 || args[1] != "glob" {
            println!("{}", USAGE);
            return Ok(());
        }

        (true, &args[2])
    };

    if glob {
        // collect everything immediately so new files don't intervene
        let raw_entries: Vec<_> = glob::glob(pattern)
            .with_context(|_| "Invalid glob pattern")?
            .into_iter()
            .try_collect()
            .with_context(|_| "Error during glob matching")?;

        // map entries to paths to consider
        let mut entries = vec![];
        for path in raw_entries {
            if path.extension() == Some(OsStr::new("srt")) {
                // assume srt files actually exist
                entries.push(path);
            } else {
                // only change extension if the srt file actually exists
                let path_srt = with_str_ext(&path);
                if path_srt
                    .try_exists()
                    .context("Error while checking if file exists")?
                {
                    entries.push(path_srt);
                }
            }
        }

        // remove duplicate entries
        entries.dedup();

        // clean each entry
        for entry in entries {
            clean_single(dry, entry)?;
        }
    } else {
        // here we assume the file exists, since the user manually specified it
        clean_single(dry, with_str_ext(pattern))?;
    }

    Ok(())
}

fn with_str_ext(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref().with_extension("srt")
}

fn clean_single(dry: bool, path: impl AsRef<Path>) -> Result<(), Error> {
    let input_path = path.as_ref();
    assert!(input_path.extension().map_or(false, |s| s == "srt"));
    println!("Cleaning {:?}", input_path);

    let mut input_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open(&input_path)
        .with_context(|_| format!("Could not open input file {:?}", input_path))?;
    let mut input_content = String::new();
    input_file
        .read_to_string(&mut input_content)
        .context("Could not read input")?;

    //ensure the input starts without whitespace and ends with two newlines
    //TODO find a way to do this without copying the string
    let mut input_content = input_content.trim().to_string();
    input_content += "\n\n";

    //TODO maybe move this to the parser
    let input_content = input_content.replace("\r\n", "\n");
    let mut subtitle = Subtitle::parse(&input_content).context("Failed to parse subtitle")?;
    clean_subtitle(&mut subtitle);
    let new_content = subtitle.to_string();

    let changed = new_content.trim() != input_content.trim();
    if dry {
        println!("Changed: {}", changed);
        return Ok(());
    }

    if changed {
        // backup to old file
        let old_path = input_path.with_extension("srt.old");
        let mut old_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&old_path)
            .context("Could not open .old file")?;
        old_file
            .write_all(input_content.as_bytes())
            .context("Error while writing .old file")?;

        // modify input file
        input_file
            .seek(SeekFrom::Start(0))
            .context("Error while seeking output")?;
        input_file
            .set_len(new_content.as_bytes().len() as u64)
            .context("Error while setting output file size")?;
        input_file
            .write_all(new_content.as_bytes())
            .context("Error while writing output")?;
    }

    Ok(())
}
