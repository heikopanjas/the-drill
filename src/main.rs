use std::{fs::File, path::PathBuf};

use clap::Parser;

use crate::cli::{Cli, Commands, DebugOptions};

mod cli;
mod dissector_builder;
mod hexdump;
mod id3v2_3_dissector;
mod id3v2_4_dissector;
mod id3v2_attached_picture_frame;
mod id3v2_chapter_frame;
mod id3v2_comment_frame;
mod id3v2_frame;
mod id3v2_table_of_contents_frame;
mod id3v2_text_encoding;
mod id3v2_text_frame;
mod id3v2_tools;
mod id3v2_unique_file_id_frame;
mod id3v2_url_frame;
mod id3v2_user_text_frame;
mod id3v2_user_url_frame;
mod isobmff_box;
mod isobmff_chapter;
mod isobmff_content;
mod isobmff_data_reference;
mod isobmff_dissector;
mod isobmff_edit_list;
mod isobmff_file_type;
mod isobmff_handler;
mod isobmff_media_header;
mod isobmff_media_info_header;
mod isobmff_metadata_keys;
mod isobmff_movie_header;
mod isobmff_sample_table;
mod isobmff_track_header;
mod itunes_metadata;
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
