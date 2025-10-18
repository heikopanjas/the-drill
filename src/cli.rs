use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "supertool")]
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
    /// Debug and analyze ID3v2 media files
    Debug
    {
        /// Path to the media file to analyze
        file: PathBuf,

        /// Show only ID3v2 header information
        #[arg(long)]
        header: bool,

        /// Show only ID3v2 frame information
        #[arg(long)]
        frames: bool,

        /// Show both header and frames (default if no options specified)
        #[arg(long)]
        all: bool
    }
}

/// Options for controlling debug output
#[derive(Debug, Clone)]
pub struct DebugOptions
{
    pub show_header: bool,
    pub show_frames: bool
}

impl DebugOptions
{
    pub fn from_flags(header: bool, frames: bool, all: bool) -> Self
    {
        // If no flags specified, default to showing everything
        if !header && !frames && !all
        {
            return DebugOptions { show_header: true, show_frames: true };
        }

        // If --all is specified, show everything regardless of other flags
        if all
        {
            return DebugOptions { show_header: true, show_frames: true };
        }

        // Otherwise, use the specific flags
        DebugOptions { show_header: header, show_frames: frames }
    }
}
