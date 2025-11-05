use std::{fs::File, path::PathBuf};

use clap::Parser;

use crate::cli::{Cli, Commands, DebugOptions};

mod cli;
mod dissector_builder;
mod hexdump;
mod id3v2;
mod isobmff;
mod media_dissector;
mod unknown_dissector;

use dissector_builder::DissectorBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let cli = Cli::parse();

    match cli.command
    {
        | Commands::Debug { file, header, data, all, verbose, dump } =>
        {
            let options = DebugOptions::from_flags(header, data, all, verbose, dump);
            dissect_file(&file, &options)?;
        }
    }

    Ok(())
}

fn dissect_file(file_path: &PathBuf, options: &DebugOptions) -> Result<(), Box<dyn std::error::Error>>
{
    // Open file
    let mut file = File::open(file_path)?;

    // Build appropriate dissector based on file content
    let builder = DissectorBuilder::new();
    let dissector = builder.build_for_file(&mut file)?;

    // Print file info
    println!("Analyzing file: {}", file_path.display());
    println!("Detected format: {} ({})", dissector.media_type(), dissector.name());

    // Perform dissection with options
    dissector.dissect_with_options(&mut file, options)?;

    Ok(())
}
