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
        all: bool
    }
}

/// Options for controlling debug output
#[derive(Debug, Clone)]
pub struct DebugOptions
{
    pub show_header: bool,
    pub show_data:   bool
}

impl DebugOptions
{
    pub fn from_flags(header: bool, data: bool, all: bool) -> Self
    {
        // If no flags specified, default to showing everything
        if !header && !data && !all
        {
            return DebugOptions { show_header: true, show_data: true };
        }

        // If --all is specified, show everything regardless of other flags
        if all
        {
            return DebugOptions { show_header: true, show_data: true };
        }

        // Otherwise, use the specific flags
        DebugOptions { show_header: header, show_data: data }
    }
}
