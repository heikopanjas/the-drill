use std::{
    fmt,
    fs::File,
    io::{Read, Seek, SeekFrom}
};

use owo_colors::OwoColorize;

use crate::{
    cli::DebugOptions,
    isobmff::{r#box::IsobmffBox, content::*, itunes_metadata::ItunesMetadata},
    media_dissector::MediaDissector
};

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
        if let Some(ref itunes_content) = self.itunes_content
        {
            let content_str = format!("{}", itunes_content);
            for line in content_str.lines()
            {
                writeln!(f, "{}    {}", indent_str, line)?;
            }
        }

        // Display parsed content for standard ISOBMFF boxes
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

                // Special handling for FullBox containers - they have version/flags (4 bytes) before children
                // meta: just version/flags
                // dref: version/flags + entry_count (8 bytes total)
                if isobmff_box.box_type == "meta" && content_end - content_start >= 4
                {
                    content_start += 4; // Skip version (1 byte) + flags (3 bytes)
                }
                else if isobmff_box.box_type == "dref" && content_end - content_start >= 8
                {
                    content_start += 8; // Skip version/flags (4 bytes) + entry_count (4 bytes)
                }

                isobmff_box.children = Self::parse_boxes(file, content_start, content_end, depth + 1)?;

                // Parse iTunes metadata if this is a metadata box with a 'data' child
                if Self::is_itunes_metadata_box(&box_type)
                {
                    // Look for 'data' child box
                    if let Some(data_box) = isobmff_box.children.iter().find(|child| child.box_type == "data") &&
                        !data_box.data.is_empty()
                    {
                        match ItunesMetadata::parse(&box_type, &data_box.data)
                        {
                            | Ok(metadata) => isobmff_box.itunes_content = Some(metadata),
                            | Err(_) =>
                            {} // Ignore parsing errors for now
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

                    // Parse content for standard ISOBMFF boxes
                    isobmff_box.content = match box_type.as_str()
                    {
                        | "ftyp" => FileTypeBox::parse(&isobmff_box.data).ok().map(IsobmffContent::FileType),
                        | "mvhd" => MovieHeaderBox::parse(&isobmff_box.data).ok().map(IsobmffContent::MovieHeader),
                        | "tkhd" => TrackHeaderBox::parse(&isobmff_box.data).ok().map(IsobmffContent::TrackHeader),
                        | "mdhd" => MediaHeaderBox::parse(&isobmff_box.data).ok().map(IsobmffContent::MediaHeader),
                        | "hdlr" => HandlerBox::parse(&isobmff_box.data).ok().map(IsobmffContent::Handler),
                        | "vmhd" => VideoMediaHeaderBox::parse(&isobmff_box.data).ok().map(IsobmffContent::VideoMediaHeader),
                        | "smhd" => SoundMediaHeaderBox::parse(&isobmff_box.data).ok().map(IsobmffContent::SoundMediaHeader),
                        | "nmhd" => NullMediaHeaderBox::parse(&isobmff_box.data).ok().map(IsobmffContent::NullMediaHeader),
                        | "dref" => DataReferenceBox::parse(&isobmff_box.data).ok().map(IsobmffContent::DataReference),
                        | "stsd" => SampleDescriptionBox::parse(&isobmff_box.data).ok().map(IsobmffContent::SampleDescription),
                        | "stts" => TimeToSampleBox::parse(&isobmff_box.data).ok().map(IsobmffContent::TimeToSample),
                        | "stsc" => SampleToChunkBox::parse(&isobmff_box.data).ok().map(IsobmffContent::SampleToChunk),
                        | "stsz" => SampleSizeBox::parse(&isobmff_box.data).ok().map(IsobmffContent::SampleSize),
                        | "stco" => ChunkOffsetBox::parse(&isobmff_box.data).ok().map(IsobmffContent::ChunkOffset),
                        | "co64" => ChunkOffset64Box::parse(&isobmff_box.data).ok().map(IsobmffContent::ChunkOffset64),
                        | "elst" => EditListBox::parse(&isobmff_box.data).ok().map(IsobmffContent::EditList),
                        | "url " => UrlEntryBox::parse(&isobmff_box.data).ok().map(IsobmffContent::UrlEntry),
                        | "urn " => UrnEntryBox::parse(&isobmff_box.data).ok().map(IsobmffContent::UrnEntry),
                        | "chap" => ChapterBox::parse(&isobmff_box.data).ok().map(IsobmffContent::Chapter),
                        | "mean" => MetadataMeanBox::parse(&isobmff_box.data).ok().map(IsobmffContent::MetadataMean),
                        | "name" => MetadataNameBox::parse(&isobmff_box.data).ok().map(IsobmffContent::MetadataName),
                        | _ => None
                    };
                }
            }

            boxes.push(isobmff_box);
            current_offset += box_size;
        }

        Ok(boxes)
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
