//! YAML2EDID tool

use std::{fs::File, io::Write as _, path::PathBuf};

use anyhow::{Context as _, Result};
use clap::Parser;
use redid::{hdmi::HdmiEdid, EdidRelease3, EdidRelease4, IntoBytes as _};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "variant")]
enum InputEdid {
    #[serde(rename = "1.3")]
    Release3(EdidRelease3),
    #[serde(rename = "1.4")]
    Release4(EdidRelease4),
    #[serde(rename = "hdmi")]
    Hdmi(HdmiEdid),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputFile {
    edid: InputEdid,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    file: PathBuf,
}

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    let args = Args::parse();

    let file = File::open(&args.file).context("Couldn't open EDID description file.")?;

    let input: InputFile =
        serde_yaml::from_reader(&file).context("Couldn't parse the EDID description file.")?;

    stdout
        .write_all(&match input.edid {
            InputEdid::Hdmi(e) => e.into_bytes(),
            InputEdid::Release3(e) => e.into_bytes(),
            InputEdid::Release4(e) => e.into_bytes(),
        })
        .context("Couldn't output our EDID binary.")?;
    stdout.flush().context("Couldn't flush stdout.")?;

    Ok(())
}
