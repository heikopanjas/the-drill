// ISO Base Media File Format (ISOBMFF) dissection
//
// This module provides comprehensive support for ISOBMFF containers including
// MP4, MOV, M4A, M4V, 3GP and other formats based on ISO/IEC 14496-12.
// Supports hierarchical box parsing, iTunes metadata, and standard box types.

// Core types and dissector
pub mod r#box;
pub mod content;
pub mod dissector;
pub mod itunes_metadata;

// Box type implementations
pub mod boxes
{
    pub mod chapter;
    pub mod data_reference;
    pub mod edit_list;
    pub mod file_type;
    pub mod handler;
    pub mod media_header;
    pub mod media_info_header;
    pub mod metadata_keys;
    pub mod movie_header;
    pub mod sample_table;
    pub mod track_header;
}

// Re-export commonly used types for convenience
pub use dissector::IsobmffDissector;
