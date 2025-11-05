use std::{
    fmt,
    fs::File,
    io::{Read, Seek, SeekFrom}
};

use owo_colors::OwoColorize;

use crate::{cli::DebugOptions, itunes_metadata::ItunesMetadata, media_dissector::MediaDissector};

/// Represents an ISOBMFF box (also called "atom")
#[derive(Debug, Clone)]
pub struct IsobmffBox
{
    pub offset:       u64,
    pub box_type:     String,
    pub size:         u64,
    pub header_size:  u64,
    pub is_container: bool,
    pub children:     Vec<IsobmffBox>,
    pub data:         Vec<u8>,
    pub content:      Option<ItunesMetadata>
}

impl IsobmffBox
{
    /// Create a new ISOBMFF box
    pub fn new(offset: u64, box_type: String, size: u64, header_size: u64) -> Self
    {
        let is_container = Self::is_container_type(&box_type);

        Self { offset, box_type, size, header_size, is_container, children: Vec::new(), data: Vec::new(), content: None }
    }

    /// Check if a box type is a container
    fn is_container_type(box_type: &str) -> bool
    {
        // Standard containers
        if matches!(
            box_type,
            "moov" | "trak" | "edts" | "mdia" | "minf" | "dinf" | "stbl" | "mvex" | "moof" | "traf" | "mfra" | "meta" | "ipro" | "udta" | "tref" | "ilst"
        )
        {
            return true;
        }

        // iTunes metadata boxes are also containers (contain 'data' child)
        box_type.starts_with('©') ||
            matches!(
                box_type,
                "trkn" |
                    "disk" |
                    "tmpo" |
                    "covr" |
                    "aART" |
                    "----" |
                    "gnre" |
                    "hdvd" |
                    "pgap" |
                    "pcst" |
                    "cpil" |
                    "rtng" |
                    "stik" |
                    "tven" |
                    "tves" |
                    "tvnn" |
                    "tvsh" |
                    "tvsn" |
                    "apID" |
                    "akID" |
                    "atID" |
                    "cnID" |
                    "geID" |
                    "plID" |
                    "sfID" |
                    "soaa" |
                    "soal" |
                    "soar" |
                    "soco" |
                    "sonm" |
                    "sosn" |
                    "xid " |
                    "keyw" |
                    "catg" |
                    "purl" |
                    "egid" |
                    "desc" |
                    "ldes" |
                    "sdes"
            )
    }

    /// Get human-readable description of box type
    pub fn get_description(&self) -> &'static str
    {
        get_box_description(&self.box_type)
    }

    /// Get the data size (excluding header)
    pub fn data_size(&self) -> u64
    {
        self.size.saturating_sub(self.header_size)
    }
}

/// Wrapper for displaying box with verbose option
pub struct VerboseBoxDisplay<'a>
{
    pub box_ref:   &'a IsobmffBox,
    pub verbose:   bool,
    pub show_dump: bool
}

impl<'a> fmt::Display for VerboseBoxDisplay<'a>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        self.box_ref.fmt_with_indent_and_options(f, 0, self.verbose, self.show_dump)
    }
}

impl fmt::Display for IsobmffBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        self.fmt_with_indent(f, 0)
    }
}

impl IsobmffBox
{
    fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result
    {
        self.fmt_with_indent_and_options(f, indent, false, false)
    }

    fn fmt_with_indent_and_options(&self, f: &mut fmt::Formatter<'_>, indent: usize, verbose: bool, show_dump: bool) -> fmt::Result
    {
        // Skip certain technical boxes unless verbose mode is enabled
        if !verbose && matches!(self.box_type.as_str(), "mdat" | "free" | "stts" | "stsc" | "stsz" | "stco" | "co64")
        {
            return Ok(());
        }

        let indent_str = "    ".repeat(indent);

        // Format box display string
        let box_info = format!("'{}' ({})", self.box_type, self.get_description());

        // Color code based on box type
        if self.is_container
        {
            writeln!(f, "{}Box at offset 0x{:08X}: {} - Size: {} bytes", indent_str, self.offset, box_info.cyan(), self.size)?;
        }
        else if matches!(self.box_type.as_str(), "ftyp" | "mdat")
        {
            writeln!(f, "{}Box at offset 0x{:08X}: {} - Size: {} bytes", indent_str, self.offset, box_info.yellow(), self.size)?;
        }
        else
        {
            writeln!(f, "{}Box at offset 0x{:08X}: {} - Size: {} bytes", indent_str, self.offset, box_info, self.size)?;
        }

        // Display parsed content for iTunes metadata boxes
        if let Some(ref content) = self.content
        {
            let content_str = format!("{}", content);
            for line in content_str.lines()
            {
                writeln!(f, "{}    {}", indent_str, line)?;
            }
        }

        // Show hexdump if requested and box has data
        if show_dump && !self.data.is_empty()
        {
            writeln!(f, "{}    Raw data:", indent_str)?;
            // Limit hexdump for covr boxes (cover art) and large data boxes (likely images) to 128 bytes
            // Data boxes > 1KB are likely image data inside covr/artwork containers
            let hexdump = if self.box_type == "covr" || (self.box_type == "data" && self.data.len() > 1024)
            {
                crate::hexdump::format_hexdump_limited(&self.data, 0, Some(128))
            }
            else
            {
                crate::hexdump::format_hexdump(&self.data, 0)
            };
            for line in hexdump.lines()
            {
                writeln!(f, "{}    {}", indent_str, line)?;
            }
            writeln!(f)?;
        }

        // Display children for container boxes
        if self.is_container && !self.children.is_empty()
        {
            for child in &self.children
            {
                child.fmt_with_indent_and_options(f, indent + 1, verbose, show_dump)?;
            }
        }

        Ok(())
    }
}

/// ISOBMFF (ISO Base Media File Format) dissector - unit struct
pub struct IsobmffDissector;

impl IsobmffDissector
{
    /// Convert box type bytes to string, handling MacRoman encoding
    /// In iTunes metadata, 0xA9 (MacRoman ©) is replaced with '@' for display
    fn box_type_to_string(bytes: &[u8]) -> String
    {
        bytes
            .iter()
            .map(|&b| {
                if b == 0xA9
                {
                    '©' // Keep the © symbol
                }
                else if b.is_ascii_graphic() || b == b' '
                {
                    b as char
                }
                else
                {
                    '?' // Replace other non-printable characters
                }
            })
            .collect()
    }

    /// Check if a box is an iTunes metadata box (should have 'data' child)
    fn is_itunes_metadata_box(box_type: &str) -> bool
    {
        // iTunes metadata boxes: text boxes with ©, other known metadata boxes
        box_type.starts_with('©') ||
            matches!(
                box_type,
                "trkn" |
                    "disk" |
                    "tmpo" |
                    "covr" |
                    "aART" |
                    "----" |
                    "gnre" |
                    "hdvd" |
                    "pgap" |
                    "pcst" |
                    "cpil" |
                    "rtng" |
                    "stik" |
                    "tven" |
                    "tves" |
                    "tvnn" |
                    "tvsh" |
                    "tvsn" |
                    "apID" |
                    "akID" |
                    "atID" |
                    "cnID" |
                    "geID" |
                    "plID" |
                    "sfID" |
                    "soaa" |
                    "soal" |
                    "soar" |
                    "soco" |
                    "sonm" |
                    "sosn" |
                    "xid " |
                    "keyw" |
                    "catg" |
                    "purl" |
                    "egid" |
                    "desc" |
                    "ldes" |
                    "sdes"
            )
    }

    /// Parse boxes from file
    fn parse_boxes(file: &mut File, start_offset: u64, end_offset: u64, depth: usize) -> Result<Vec<IsobmffBox>, String>
    {
        let mut boxes = Vec::new();
        let mut current_offset = start_offset;

        // Prevent excessive recursion
        if depth > 20
        {
            return Err("Maximum box nesting depth exceeded".to_string());
        }

        while current_offset < end_offset
        {
            file.seek(SeekFrom::Start(current_offset)).map_err(|e| format!("Seek error at offset 0x{:08X}: {}", current_offset, e))?;

            // Read box header (minimum 8 bytes: 4 for size, 4 for type)
            let mut header = [0u8; 8];
            file.read_exact(&mut header).map_err(|e| format!("Failed to read box header at 0x{:08X}: {}", current_offset, e))?;

            // Parse size and type
            let size_32 = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
            let box_type = Self::box_type_to_string(&header[4..8]);

            let (box_size, header_size) = if size_32 == 1
            {
                // Extended size (64-bit)
                let mut extended_size = [0u8; 8];
                file.read_exact(&mut extended_size).map_err(|e| format!("Failed to read extended size: {}", e))?;
                let size_64 = u64::from_be_bytes(extended_size);
                (size_64, 16u64)
            }
            else if size_32 == 0
            {
                // Box extends to end of file
                (end_offset - current_offset, 8u64)
            }
            else
            {
                (size_32 as u64, 8u64)
            };

            // Validate box size
            if box_size < header_size
            {
                return Err(format!("Invalid box size {} at offset 0x{:08X} (smaller than header)", box_size, current_offset));
            }

            if current_offset + box_size > end_offset
            {
                return Err(format!("Box at offset 0x{:08X} extends beyond parent (size: {}, available: {})", current_offset, box_size, end_offset - current_offset));
            }

            let mut isobmff_box = IsobmffBox::new(current_offset, box_type.clone(), box_size, header_size);

            // Parse container contents or read data
            if isobmff_box.is_container
            {
                let mut content_start = current_offset + header_size;
                let content_end = current_offset + box_size;

                // Special handling for 'meta' box - it has version/flags (4 bytes) before children
                if isobmff_box.box_type == "meta" && content_end - content_start >= 4
                {
                    content_start += 4; // Skip version (1 byte) + flags (3 bytes)
                }

                isobmff_box.children = Self::parse_boxes(file, content_start, content_end, depth + 1)?;

                // Parse iTunes metadata if this is a metadata box with a 'data' child
                if Self::is_itunes_metadata_box(&box_type)
                {
                    // Look for 'data' child box
                    if let Some(data_box) = isobmff_box.children.iter().find(|child| child.box_type == "data")
                    {
                        if !data_box.data.is_empty()
                        {
                            match ItunesMetadata::parse(&box_type, &data_box.data)
                            {
                                | Ok(metadata) => isobmff_box.content = Some(metadata),
                                | Err(_) =>
                                {} // Ignore parsing errors for now
                            }
                        }
                    }
                }
            }
            else
            {
                // Read box data for leaf boxes (but limit very large boxes like mdat)
                let data_size = isobmff_box.data_size();

                // Only read data for smaller boxes (skip large media data)
                if data_size > 0 && data_size <= 1024 * 1024
                {
                    file.seek(SeekFrom::Start(current_offset + header_size)).map_err(|e| format!("Seek error: {}", e))?;

                    let mut data = vec![0u8; data_size as usize];
                    file.read_exact(&mut data).map_err(|e| format!("Failed to read box data: {}", e))?;

                    isobmff_box.data = data;
                }
            }

            boxes.push(isobmff_box);
            current_offset += box_size;
        }

        Ok(boxes)
    }

    /// Parse ftyp box details
    fn format_ftyp_details(ftyp: &IsobmffBox) -> String
    {
        let mut output = String::new();

        if ftyp.data.len() >= 8
        {
            let major_brand = String::from_utf8_lossy(&ftyp.data[0..4]);
            let minor_version = u32::from_be_bytes([ftyp.data[4], ftyp.data[5], ftyp.data[6], ftyp.data[7]]);

            output.push_str(&format!("    Major Brand: '{}'\n", major_brand));
            output.push_str(&format!("    Minor Version: {}\n", minor_version));

            if ftyp.data.len() > 8
            {
                output.push_str("    Compatible Brands: ");
                let mut brands = Vec::new();
                for chunk in ftyp.data[8..].chunks(4)
                {
                    if chunk.len() == 4
                    {
                        brands.push(format!("'{}'", String::from_utf8_lossy(chunk)));
                    }
                }
                output.push_str(&brands.join(", "));
                output.push('\n');
            }
        }

        output
    }
}

impl MediaDissector for IsobmffDissector
{
    fn media_type(&self) -> &'static str
    {
        "ISOBMFF"
    }

    fn name(&self) -> &'static str
    {
        "ISO Base Media File Format Dissector"
    }

    fn dissect_with_options(&self, file: &mut File, options: &DebugOptions) -> Result<(), Box<dyn std::error::Error>>
    {
        let file_size = file.metadata()?.len();

        // Parse all boxes
        let boxes = Self::parse_boxes(file, 0, file_size, 0).map_err(|e| format!("Failed to parse ISOBMFF boxes: {}", e))?;

        // Header information
        if options.show_header
        {
            println!("\n{}", "ISO Base Media File Format Header:".bright_cyan().bold());

            // Show ftyp box if present
            if let Some(ftyp) = boxes.first() &&
                ftyp.box_type == "ftyp"
            {
                println!("{}", ftyp);
                print!("{}", Self::format_ftyp_details(ftyp));
            }

            println!();
        }

        // Boxes/structure information
        if options.show_data
        {
            println!("{}\n", "Box Structure:".bright_cyan().bold());

            for isobmff_box in &boxes
            {
                print!("{}", VerboseBoxDisplay { box_ref: isobmff_box, verbose: options.show_verbose, show_dump: options.show_dump });
            }
        }

        Ok(())
    }

    fn can_handle(&self, header: &[u8]) -> bool
    {
        // Need at least 12 bytes to check for ftyp box
        if header.len() < 12
        {
            return false;
        }

        // Check for 'ftyp' box at start of file
        let box_type = String::from_utf8_lossy(&header[4..8]);

        if box_type == "ftyp"
        {
            // Additional validation: check major brand
            let major_brand = String::from_utf8_lossy(&header[8..12]);

            // Common ISOBMFF brands
            let valid_brands = [
                "isom", "iso2", "iso3", "iso4", "iso5", "iso6", "mp41", "mp42", "mp71", "M4A ", "M4V ", "M4P ", "M4B ", "qt  ", "mqt ", "3gp4", "3gp5", "3gp6",
                "3gp7", "3gp8", "3gp9", "3g2a", "3g2b", "3g2c", "mmp4", "avc1", "iso5", "MSNV", "dash", "msdh", "msix"
            ];

            return valid_brands.iter().any(|&b| major_brand == b);
        }

        false
    }
}

/// Get human-readable description for box types
fn get_box_description(box_type: &str) -> &'static str
{
    match box_type
    {
        // File-level boxes
        | "ftyp" => "File Type and Compatibility",
        | "moov" => "Movie Metadata Container",
        | "mdat" => "Media Data",
        | "free" => "Free Space",
        | "skip" => "Free Space",
        | "moof" => "Movie Fragment",
        | "mfra" => "Movie Fragment Random Access",
        | "meta" => "Metadata Container",
        | "pdin" => "Progressive Download Information",
        | "styp" => "Segment Type",
        | "sidx" => "Segment Index",

        // Movie box children
        | "mvhd" => "Movie Header",
        | "trak" => "Track Container",
        | "mvex" => "Movie Extends",
        | "udta" => "User Data",
        | "iods" => "Initial Object Descriptor",

        // Track box children
        | "tkhd" => "Track Header",
        | "tref" => "Track Reference",
        | "edts" => "Edit List Container",
        | "mdia" => "Media Container",

        // Track reference box children
        | "chap" => "Chapter Track Reference",
        | "tmcd" => "Timecode Track Reference",
        | "sync" => "Sync Track Reference",
        | "scpt" => "Script Track Reference",
        | "ssrc" => "Non-Primary Source",
        | "cdsc" => "Content Description Track Reference",

        // Edit box children
        | "elst" => "Edit List",

        // Media box children
        | "mdhd" => "Media Header",
        | "hdlr" => "Handler Reference",
        | "minf" => "Media Information",

        // Media information box children
        | "vmhd" => "Video Media Header",
        | "smhd" => "Sound Media Header",
        | "hmhd" => "Hint Media Header",
        | "nmhd" => "Null Media Header",
        | "dinf" => "Data Information",
        | "stbl" => "Sample Table",

        // Data information box children
        | "dref" => "Data Reference",
        | "url " => "Data Entry URL",
        | "urn " => "Data Entry URN",

        // Sample table box children
        | "stsd" => "Sample Description",
        | "stts" => "Time-to-Sample",
        | "ctts" => "Composition Time-to-Sample",
        | "stsc" => "Sample-to-Chunk",
        | "stsz" => "Sample Sizes",
        | "stz2" => "Compact Sample Sizes",
        | "stco" => "Chunk Offset (32-bit)",
        | "co64" => "Chunk Offset (64-bit)",
        | "stss" => "Sync Sample Table",
        | "stsh" => "Shadow Sync Sample",
        | "padb" => "Padding Bits",
        | "stdp" => "Sample Degradation Priority",
        | "sdtp" => "Sample Dependency",
        | "sbgp" => "Sample-to-Group",
        | "sgpd" => "Sample Group Description",
        | "subs" => "Sub-Sample Information",

        // Movie extends box children
        | "mehd" => "Movie Extends Header",
        | "trex" => "Track Extends Defaults",
        | "leva" => "Level Assignment",

        // Movie fragment box children
        | "mfhd" => "Movie Fragment Header",
        | "traf" => "Track Fragment",

        // Track fragment box children
        | "tfhd" => "Track Fragment Header",
        | "trun" => "Track Fragment Run",
        | "tfdt" => "Track Fragment Decode Time",

        // Movie fragment random access box children
        | "tfra" => "Track Fragment Random Access",
        | "mfro" => "Movie Fragment Random Access Offset",

        // Metadata box children
        | "iloc" => "Item Location",
        | "ipro" => "Item Protection",
        | "iinf" => "Item Information",
        | "xml " => "XML Metadata",
        | "bxml" => "Binary XML Metadata",
        | "pitm" => "Primary Item",
        | "idat" => "Item Data",
        | "iref" => "Item Reference",

        // User data box children
        | "cprt" => "Copyright",
        | "name" => "Name",
        | "©nam" => "Name (iTunes)",
        | "©ART" => "Artist (iTunes)",
        | "©alb" => "Album (iTunes)",
        | "©day" => "Year (iTunes)",
        | "©cmt" => "Comment (iTunes)",
        | "©gen" => "Genre (iTunes)",
        | "©too" => "Encoding Tool (iTunes)",
        | "©wrt" => "Composer (iTunes)",
        | "©grp" => "Grouping (iTunes)",
        | "©lyr" => "Lyrics (iTunes)",
        | "trkn" => "Track Number (iTunes)",
        | "disk" => "Disk Number (iTunes)",
        | "tmpo" => "Tempo (iTunes)",
        | "covr" => "Cover Art (iTunes)",
        | "aART" => "Album Artist (iTunes)",
        | "----" => "Custom iTunes Metadata",
        | "ilst" => "iTunes Metadata List",
        | "mean" => "iTunes Metadata Mean",
        | "data" => "iTunes Metadata Data",
        | "keyw" => "Keywords",
        | "catg" => "Category",
        | "purl" => "Podcast URL",
        | "egid" => "Episode Global Unique ID",
        | "desc" => "Description",
        | "ldes" => "Long Description",
        | "sdes" => "Short Description",

        // Sample description entries (video)
        | "avc1" => "AVC/H.264 Video",
        | "avc2" => "AVC/H.264 Video (parameter sets in-band)",
        | "avc3" => "AVC/H.264 Video (no parameter sets)",
        | "avc4" => "AVC/H.264 Video (parameter sets in-band, no SPS/PPS)",
        | "hvc1" => "HEVC/H.265 Video",
        | "hev1" => "HEVC/H.265 Video (parameter sets in-band)",
        | "mp4v" => "MPEG-4 Visual",
        | "s263" => "H.263 Video",
        | "vp08" => "VP8 Video",
        | "vp09" => "VP9 Video",
        | "av01" => "AV1 Video",
        | "dvh1" => "Dolby Vision H.265",
        | "dvhe" => "Dolby Vision H.265 (profile 8)",
        | "mjp2" => "Motion JPEG 2000",

        // Sample description entries (audio)
        | "mp4a" => "MPEG-4 Audio (AAC)",
        | "samr" => "AMR Narrow-Band Audio",
        | "sawb" => "AMR Wide-Band Audio",
        | "sawp" => "AMR Wide-Band+ Audio",
        | "ac-3" => "AC-3 Audio (Dolby Digital)",
        | "ec-3" => "Enhanced AC-3 Audio (Dolby Digital Plus)",
        | "dtsc" => "DTS Coherent Acoustics",
        | "dtsh" => "DTS-HD High Resolution",
        | "dtsl" => "DTS-HD Master Audio",
        | "dtse" => "DTS Express",
        | "alac" => "Apple Lossless Audio",
        | "fLaC" => "FLAC Audio",
        | "Opus" => "Opus Audio",
        | "mp3 " => "MPEG-1/2 Audio Layer III",
        | "alaw" => "A-law Audio",
        | "ulaw" => "μ-law Audio",
        | "sowt" => "PCM Signed Little-Endian",
        | "twos" => "PCM Signed Big-Endian",
        | "raw " => "PCM Uncompressed",
        | "lpcm" => "Linear PCM",

        // Sample description entries (text/subtitle)
        | "tx3g" => "3GPP Timed Text",
        | "text" => "QuickTime Text",
        | "wvtt" => "WebVTT Subtitle",
        | "stpp" => "XML Subtitle",
        | "c608" => "CEA-608 Closed Captions",
        | "c708" => "CEA-708 Closed Captions",

        // Sample description entries (metadata)
        | "mett" => "Metadata Text",
        | "metx" => "Metadata XML",
        | "urim" => "URI Metadata",

        // Protection/encryption boxes
        | "sinf" => "Protection Scheme Information",
        | "frma" => "Original Format",
        | "schm" => "Scheme Type",
        | "schi" => "Scheme Information",
        | "encv" => "Encrypted Video Sample Entry",
        | "enca" => "Encrypted Audio Sample Entry",
        | "enct" => "Encrypted Text Sample Entry",

        // Additional container boxes
        | "rinf" => "Restricted Scheme Information",
        | "trgr" => "Track Grouping",
        | "grpl" => "Group List",

        // QuickTime specific
        | "wide" => "QuickTime Wide Atom (deprecated)",
        | "pnot" => "Preview",
        | "clip" => "Clipping",
        | "crgn" => "Clipping Region",
        | "matt" => "Matte",
        | "kmat" => "Compressed Matte",
        | "load" => "Track Load Settings",
        | "imap" => "Track Input Map",
        | "uuid" => "User Extension (UUID)",

        // Additional audio/video configuration boxes
        | "esds" => "MPEG-4 Elementary Stream Descriptor",
        | "avcC" => "AVC Configuration",
        | "hvcC" => "HEVC Configuration",
        | "vpcC" => "VP Codec Configuration",
        | "av1C" => "AV1 Configuration",
        | "dac3" => "AC-3 Specific Box",
        | "dec3" => "Enhanced AC-3 Specific Box",
        | "dvc1" => "VC-1 Configuration",
        | "btrt" => "Bit Rate",
        | "colr" => "Color Information",
        | "pasp" => "Pixel Aspect Ratio",
        | "clap" => "Clean Aperture",
        | "mdcv" => "Mastering Display Color Volume",
        | "clli" => "Content Light Level",
        | "fiel" => "Field/Frame Information",

        // Additional iTunes metadata boxes
        | "©cpy" => "Copyright (iTunes)",
        | "©dir" => "Director (iTunes)",
        | "©ed1" => "Edit Date 1 (iTunes)",
        | "©ed2" => "Edit Date 2 (iTunes)",
        | "©ed3" => "Edit Date 3 (iTunes)",
        | "©fmt" => "Format (iTunes)",
        | "©inf" => "Information (iTunes)",
        | "©prd" => "Producer (iTunes)",
        | "©prf" => "Performers (iTunes)",
        | "©req" => "Requirements (iTunes)",
        | "©src" => "Source (iTunes)",
        | "©swr" => "Software (iTunes)",
        | "gnre" => "Genre (iTunes old)",
        | "hdvd" => "HD Video (iTunes)",
        | "pgap" => "Gapless Playback (iTunes)",
        | "pcst" => "Podcast (iTunes)",
        | "cpil" => "Compilation (iTunes)",
        | "rtng" => "Rating (iTunes)",
        | "stik" => "Media Type (iTunes)",
        | "tven" => "TV Episode (iTunes)",
        | "tves" => "TV Episode Number (iTunes)",
        | "tvnn" => "TV Network Name (iTunes)",
        | "tvsh" => "TV Show Name (iTunes)",
        | "tvsn" => "TV Season (iTunes)",
        | "apID" => "Apple Store Account (iTunes)",
        | "akID" => "Apple Store Kind (iTunes)",
        | "atID" => "Album iTunes ID (iTunes)",
        | "cnID" => "iTunes Catalog ID (iTunes)",
        | "geID" => "Genre iTunes ID (iTunes)",
        | "plID" => "Playlist iTunes ID (iTunes)",
        | "sfID" => "Store Front ID (iTunes)",
        | "soaa" => "Sort Album Artist (iTunes)",
        | "soal" => "Sort Album (iTunes)",
        | "soar" => "Sort Artist (iTunes)",
        | "soco" => "Sort Composer (iTunes)",
        | "sonm" => "Sort Name (iTunes)",
        | "sosn" => "Sort Show (iTunes)",
        | "xid " => "Vendor ID (iTunes)",

        // Additional DASH/streaming boxes (not duplicating existing ones)
        | "tfxd" => "Track Fragment Extended Decode Time",
        | "tfrf" => "Track Fragment Reference",
        | "ssix" => "Sub-Sample Index",
        | "prft" => "Producer Reference Time",
        | "emsg" => "Event Message",

        // Default
        | _ => "Unknown Box Type"
    }
}
