# The Drill

A versatile media file analysis tool that dissects ID3v2 tags (MP3 files) and ISO Base Media File Format (ISOBMFF) containers (MP4, MOV, M4A, etc.). Built in Rust for cross-platform compatibility with a focus on detailed diagnostic output and specification compliance.

## Features

### ID3v2 Support

- **Complete ID3v2.3 and ID3v2.4 dissection** with specification compliance
- **Rich frame parsing** for all major frame types (TEXT, URL, COMM, APIC, UFID, etc.)
- **Chapter frame support** (CHAP/CTOC) from ID3v2 Chapter Frame Addendum
- **Embedded frame analysis** within chapter structures
- **Diagnostic output** with hex byte inspection and frame validation
- **Large tag handling** optimized for podcast files with chapter images (up to 100MB)

### ISOBMFF Support

- **Complete ISO Base Media File Format parsing** for MP4, MOV, M4A, M4V, 3GP, and other containers
- **Hierarchical box structure analysis** with recursive parsing (up to 20 depth levels)
- **150+ box type descriptions** including:
  - 80+ standard ISO/IEC 14496-12 boxes
  - 50+ iTunes metadata boxes with MacRoman encoding support
  - 15 video codec boxes (H.264, HEVC, VP8/9, AV1, Dolby Vision)
  - 20 audio codec boxes (AAC, Opus, FLAC, ALAC, DTS, Dolby)
  - Text/subtitle formats (3GPP, WebVTT, CEA-608/708)
  - Protection/encryption boxes
  - DASH/streaming boxes
  - QuickTime-specific boxes
- **ftyp brand detection** with validation of 25+ brand codes
- **Color-coded hierarchical display** (containers in cyan, special boxes in yellow)
- **Efficient large file handling** (skips reading media data >1MB)

### Advanced Features

- **Automatic format detection** based on file headers
- **Modular architecture** with pluggable dissector system
- **Colored diagnostic output** for enhanced readability
- **Granular output control** with `--header`, `--data`, `--verbose`, `--dump`, and `--all` options
- **Hexdump display** for low-level binary inspection
- **Technical box filtering** to focus on metadata (hides mdat, free, sample tables by default)
- **Comprehensive error reporting** with detailed validation

## Installation

### Prerequisites

- Rust 1.70+ (stable toolchain)
- Cargo package manager

### From Source

```bash
git clone https://github.com/yourusername/the-drill.git
cd the-drill
cargo build --release
```

The binary will be available at `target/release/the-drill`.

### Development Build

```bash
cargo build
```

## Usage

### Basic Analysis

```bash
# Analyze an MP3 file with full output
the-drill debug song.mp3

# Analyze an M4A/MP4 file
the-drill debug podcast.m4a

# Analyze a video file
the-drill debug movie.mp4
```

### Granular Output Control

```bash
# Show only header information (ID3v2 header or ISOBMFF ftyp)
the-drill debug --header podcast.mp3
the-drill debug --header video.mp4

# Show only data structures (frames/boxes content)
the-drill debug --data audiobook.mp3
the-drill debug --data movie.m4a

# Show everything (default)
the-drill debug --all music.mp3
the-drill debug --all audio.m4a

# Show verbose output (includes technical boxes like mdat, free, sample tables)
the-drill debug --verbose podcast.m4a

# Display hexdump of frame/box data
the-drill debug --dump song.mp3
the-drill debug --dump audio.m4a

# Combine options
the-drill debug --data --dump podcast.m4a
the-drill debug --all --verbose --dump video.mp4
```

### Command Reference

```text
the-drill debug [OPTIONS] <FILE>

Arguments:
  <FILE>  Path to the media file to analyze (MP3, MP4, M4A, MOV, M4V, 3GP, etc.)

Options:
  --header          Show only header information (ID3v2 header or ISOBMFF ftyp box)
  --data            Show only data structures (ID3v2 frames or ISOBMFF boxes)
  --all             Show both header and content (default if no options specified)
  -v, --verbose     Show verbose output including large technical boxes (mdat, free, stts, stsc, stsz, stco)
  -d, --dump        Display hexdump of frame/box data for low-level analysis
  -h, --help        Print help
  -V, --version     Print version
```

## Sample Output

### ID3v2 Analysis

```text
Analyzing file: samples/podcast.mp3
Detected format: ID3v2.4 (ID3v2.4 Dissector)

ID3v2 Header Found:
  Version: 2.4.0
  Flags: 0x00
  Tag Size: 15458392 bytes
  INFO: Large tag size (> 10MB), possibly podcast with embedded chapter content

ID3v2.4 Frames:
    Frame offset 0x0000000A, ID: [0x54, 0x49, 0x54, 0x32] = "TIT2", Size: [0x00, 0x00, 0x00, 0x2D] = 45, Flags: 0x0000
    TIT2 (Title/songname/content description) - Size: 45 bytes
        Encoding: UTF-8
        Value: "Episode 123: The Future of Technology"

    Frame offset 0x00000047, ID: [0x43, 0x48, 0x41, 0x50] = "CHAP", Size: [0x00, 0x05, 0x6C, 0xE3] = 355555, Flags: 0x0000
    CHAP (Chapter frame) - Size: 355555 bytes
        Element ID: "chapter_001"
        Time: 00:00:00.000 - 00:15:30.500 (duration: 00:15:30.500)
        Sub-frames: 3 embedded frame(s)

            Frame: TIT2 (Title/songname/content description) - Size: 28 bytes
                Encoding: UTF-8
                Value: "Introduction"

            Frame: APIC (Attached picture) - Size: 342856 bytes
                Encoding: UTF-8
                MIME type: "image/jpeg"
                Picture type: Cover (front)
                Description: "Chapter 1 Cover"
                Data size: 342825 bytes
```

### ISOBMFF Analysis

```text
Analyzing file: samples/podcast.m4a
Detected format: ISO Base Media File Format (ISOBMFF Dissector)

File Type Box (ftyp) Details:
  Major Brand: M4A
  Minor Version: 0
  Compatible Brands: M4A , mp42, isom

ISOBMFF Box Structure:
    Box: ftyp (File Type Box) - Size: 32 bytes, Offset: 0x00000000
    Box: free (Free Space) - Size: 8 bytes, Offset: 0x00000020
    Box: mdat (Media Data) - Size: 88475392 bytes, Offset: 0x00000028
        [Large mdat box - skipping data read for performance]
    Box: moov (Movie Container) - Size: 12458 bytes, Offset: 0x05459A48
        Box: mvhd (Movie Header) - Size: 108 bytes, Offset: 0x05459A50
        Box: trak (Track Container) - Size: 5234 bytes, Offset: 0x05459ABC
            Box: tkhd (Track Header) - Size: 92 bytes, Offset: 0x05459AC4
            Box: mdia (Media Container) - Size: 5134 bytes, Offset: 0x05459B20
                Box: mdhd (Media Header) - Size: 32 bytes, Offset: 0x05459B28
                Box: hdlr (Handler Reference) - Size: 45 bytes, Offset: 0x05459B48
                Box: minf (Media Information Container) - Size: 5049 bytes, Offset: 0x05459B75
                    Box: smhd (Sound Media Header) - Size: 16 bytes, Offset: 0x05459B7D
                    Box: dinf (Data Information Container) - Size: 36 bytes, Offset: 0x05459B8D
                        Box: dref (Data Reference) - Size: 28 bytes, Offset: 0x05459B95
                    Box: stbl (Sample Table Container) - Size: 4989 bytes, Offset: 0x05459BB1
                        Box: stsd (Sample Description) - Size: 103 bytes, Offset: 0x05459BB9
                        Box: stts (Time-to-Sample) - Size: 24 bytes, Offset: 0x05459C20
                        Box: stsc (Sample-to-Chunk) - Size: 28 bytes, Offset: 0x05459C38
        Box: udta (User Data Container) - Size: 2958 bytes, Offset: 0x0545A54C
            Box: meta (Metadata Container) - Size: 2950 bytes, Offset: 0x0545A554
                Box: hdlr (Handler Reference) - Size: 33 bytes, Offset: 0x0545A55C
                Box: ilst (Item List Container) - Size: 2909 bytes, Offset: 0x0545A57D
                    Box: ©nam (Name/Title) - Size: 45 bytes, Offset: 0x0545A585
                    Box: ©ART (Artist) - Size: 32 bytes, Offset: 0x0545A5B2
                    Box: ©alb (Album) - Size: 28 bytes, Offset: 0x0545A5D2
                    Box: ©gen (Genre) - Size: 22 bytes, Offset: 0x0545A5EE
                    Box: covr (Cover Art) - Size: 2734 bytes, Offset: 0x0545A604
```

### Header-Only Output

```bash
# ID3v2 file
the-drill debug --header podcast.mp3
```

```text
Analyzing file: podcast.mp3
Detected format: ID3v2.4 (ID3v2.4 Dissector)

ID3v2 Header Found:
  Version: 2.4.0
  Flags: 0x00
  Tag Size: 15458392 bytes
  INFO: Large tag size (> 10MB), possibly podcast with embedded chapter content
```

```bash
# ISOBMFF file
the-drill debug --header podcast.m4a
```

```text
Analyzing file: podcast.m4a
Detected format: ISO Base Media File Format (ISOBMFF Dissector)

File Type Box (ftyp) Details:
  Major Brand: M4A
  Minor Version: 0
  Compatible Brands: M4A , mp42, isom
```

### Hexdump Output

```bash
# Display hexdump of frame/box data
the-drill debug --dump --data song.mp3
```

```text
Analyzing file: song.mp3
Detected format: ID3v2.3 (ID3v2.3 Dissector)

ID3v2.3 Frames:
    Frame offset 0x0000000A, ID: [0x54, 0x49, 0x54, 0x32] = "TIT2", Size: [0x00, 0x00, 0x00, 0x1A] = 26, Flags: 0x0000
    TIT2 (Title/songname/content description) - Size: 26 bytes
        Encoding: UTF-8
        Value: "Amazing Song Title"

Hexdump:
00000000  01 41 6D 61 7A 69 6E 67  20 53 6F 6E 67 20 54 69  .Amazing Song Ti
00000010  74 6C 65 00                                       tle.
```

## Supported Formats

### ID3v2 Tags

- **ID3v2.3** - Complete frame parsing with big-endian integers
- **ID3v2.4** - Full support with synchsafe integers and extended features
- **Chapter Frames** - CHAP and CTOC from ID3v2 Chapter Frame Addendum
- **All standard frames** - TEXT, URL, COMM, APIC, UFID, TXXX, WXXX, etc.

### ISOBMFF Containers

- **MP4** - MPEG-4 Part 14 container files
- **M4A** - MPEG-4 Audio files (AAC, ALAC, etc.)
- **M4V** - MPEG-4 Video files
- **MOV** - QuickTime Movie files
- **3GP** - 3GPP multimedia files
- **Other ISO BMFF variants** - Any file following ISO/IEC 14496-12

### Box Types Supported

- **Container boxes** - moov, trak, mdia, minf, stbl, meta, ilst, and 10+ more
- **Standard boxes** - 80+ boxes from ISO/IEC 14496-12 specification
- **iTunes metadata** - 50+ iTunes-specific boxes (©nam, ©ART, ©alb, etc.)
- **Codec boxes** - H.264, HEVC, VP8/9, AV1, AAC, Opus, FLAC, ALAC, DTS, Dolby
- **Subtitle boxes** - 3GPP timed text, WebVTT, CEA-608/708
- **Protection boxes** - DRM and encryption boxes (sinf, encv, enca)
- **Streaming boxes** - DASH and adaptive streaming boxes

## Technical Details

### Architecture

- **Modular Design** - Pluggable dissector system with trait-based architecture
- **Format Detection** - Automatic dissector selection based on file headers
- **Memory Efficient** - Streaming analysis without loading entire files
- **Error Resilient** - Graceful handling of corrupted or non-standard files
- **Hierarchical Module Structure** - Separate `id3v2/` and `isobmff/` module trees with frame/box type modules
- **"One Struct Per File"** - Clean separation of concerns following Rust best practices

### Frame Types Supported

- **Text Frames** (T***) - All standard text information frames
- **URL Frames** (W***) - Web link frames with descriptions
- **Comment Frames** (COMM, USLT) - Comments and unsynchronized lyrics
- **Picture Frames** (APIC) - Embedded artwork with type descriptions
- **Chapter Frames** (CHAP, CTOC) - Podcast/audiobook chapter structures
- **User-Defined Frames** (TXXX, WXXX) - Custom text and URL frames
- **Unique ID Frames** (UFID) - File identification frames

### Specifications Compliance

- **ID3v2.3** - Full compliance with original specification
- **ID3v2.4** - Complete implementation including synchsafe integers
- **ID3v2 Chapter Addendum** - CHAP and CTOC frame support
- **ISO/IEC 14496-12** - ISO Base Media File Format specification
- **iTunes Metadata** - Support for Apple's proprietary metadata boxes with MacRoman encoding

## Development

### Building

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint with clippy
cargo clippy
```

## Dependencies

- **clap 4.5** - Command-line argument parsing with derive features
- **owo-colors 4.1** - Enhanced colored terminal output

### Development Guidelines

- Follow Rust best practices and idioms
- Maintain cross-platform compatibility
- Use "dissect" terminology instead of "parse"
- Add comprehensive diagnostic output
- Update documentation for new features

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- ID3v2 specification maintainers for comprehensive documentation
- ISO/IEC for the ISO Base Media File Format specification
- Apple for iTunes metadata format documentation
- Rust community for excellent tooling and libraries
- Podcast creators whose files helped test large tag handling

---

**Note**: This tool is designed for analysis and debugging purposes. It provides detailed diagnostic information about media file structures and metadata without modifying the original files.
