use crate::itunes_metadata::ItunesMetadata;

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
        let is_container = is_container_type(&box_type);

        Self { offset, box_type, size, header_size, is_container, children: Vec::new(), data: Vec::new(), content: None }
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

/// Check if a box type is a container
pub fn is_container_type(box_type: &str) -> bool
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

/// Get human-readable description for box types
pub fn get_box_description(box_type: &str) -> &'static str
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
