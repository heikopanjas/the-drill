// ID3v2 tag dissection and frame parsing
//
// This module provides comprehensive support for ID3v2.3 and ID3v2.4 tag formats,
// including all standard frame types and proper handling of text encodings,
// unsynchronization, and embedded frames in chapter structures.

// Core types and utilities
pub mod frame;
pub mod text_encoding;
pub mod tools;

// Version-specific dissectors
pub mod dissectors
{
    pub mod v3;
    pub mod v4;
}

// Frame type implementations
pub mod frames
{
    pub mod attached_picture;
    pub mod chapter;
    pub mod comment;
    pub mod table_of_contents;
    pub mod text;
    pub mod unique_file_id;
    pub mod url;
    pub mod user_text;
    pub mod user_url;
}

// Re-export commonly used types for convenience
pub use dissectors::{v3::Id3v23Dissector, v4::Id3v24Dissector};
