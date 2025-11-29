use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "the-drill")]
#[command(about = "A versatile media file analysis tool")]
#[command(version)]
pub struct Cli
{
    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand)]
pub enum Commands
{
    /// Debug and analyze media files
    Debug
    {
        /// Path to the media file to analyze
        file: PathBuf,

        /// Show only file header information
        #[arg(long)]
        header: bool,

        /// Show only data structures (ID3v2 frames, ISOBMFF boxes)
        #[arg(long)]
        data: bool,

        /// Show both header and data (default if no options specified)
        #[arg(long)]
        all: bool,

        /// Show verbose output including large technical boxes (mdat, free, stts, stsc, stsz, stco, ctts)
        #[arg(long, short)]
        verbose: bool,

        /// Show hexdump of frame/box data
        #[arg(long, short)]
        dump: bool
    }
}

/// Options for controlling debug output
#[derive(Debug, Clone)]
pub struct DebugOptions
{
    pub show_header:  bool,
    pub show_data:    bool,
    pub show_verbose: bool,
    pub show_dump:    bool
}

impl DebugOptions
{
    pub fn from_flags(header: bool, data: bool, all: bool, verbose: bool, dump: bool) -> Self
    {
        // If no flags specified, default to showing everything
        if header == false && data == false && all == false
        {
            return DebugOptions { show_header: true, show_data: true, show_verbose: verbose, show_dump: dump };
        }

        // If --all is specified, show everything regardless of other flags
        if all
        {
            return DebugOptions { show_header: true, show_data: true, show_verbose: verbose, show_dump: dump };
        }

        // Otherwise, use the specific flags
        DebugOptions { show_header: header, show_data: data, show_verbose: verbose, show_dump: dump }
    }
}
