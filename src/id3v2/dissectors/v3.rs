use std::{fs::File, io::Read};

use owo_colors::OwoColorize;

use crate::{
    cli::DebugOptions,
    id3v2::{frame::Id3v2Frame, tools::*},
    media_dissector::MediaDissector
};

/// ID3v2.3 dissector for MP3 files
pub struct Id3v23Dissector;

/// Parse an ID3v2.3 frame from raw buffer data
pub fn parse_id3v2_3_frame(buffer: &[u8], pos: usize) -> Option<Id3v2Frame>
{
    if pos + 10 > buffer.len()
    {
        return None;
    }

    let frame_id = String::from_utf8_lossy(&buffer[pos..pos + 4]).to_string();

    // Stop if we hit padding (null bytes)
    if frame_id.starts_with('\0') || !frame_id.chars().all(|c| c.is_ascii_alphanumeric())
    {
        return None;
    }

    // Check if this is a valid ID3v2.3 frame ID
    if crate::id3v2::tools::is_valid_frame_for_version(&frame_id, 3) == false
    {
        return None;
    }

    // ID3v2.3 uses regular big-endian integers (not synchsafe)
    let frame_size = u32::from_be_bytes([buffer[pos + 4], buffer[pos + 5], buffer[pos + 6], buffer[pos + 7]]);
    let frame_flags = u16::from_be_bytes([buffer[pos + 8], buffer[pos + 9]]);

    if frame_size == 0 || frame_size > (buffer.len() - pos - 10) as u32
    {
        return None;
    }

    let data = buffer[pos + 10..pos + 10 + frame_size as usize].to_vec();

    let mut frame = Id3v2Frame::new_with_offset(frame_id.clone(), frame_size, frame_flags, pos, data);

    // Parse the frame content using the new typed system (ID3v2.3)
    let _ = frame.parse_content(3); // Ignore parsing errors, keep raw data

    Some(frame)
}

impl MediaDissector for Id3v23Dissector
{
    fn media_type(&self) -> &'static str
    {
        "ID3v2.3"
    }

    fn dissect_with_options(&self, file: &mut File, options: &DebugOptions) -> Result<(), Box<dyn std::error::Error>>
    {
        dissect_id3v2_3_file_with_options(file, options)
    }

    fn can_handle(&self, header: &[u8]) -> bool
    {
        // Check for ID3v2.3 specifically
        if let Some((major, _minor)) = detect_id3v2_version(header)
        {
            return major == 3;
        }

        // Also check for MPEG sync (might contain ID3v2.3)
        detect_mpeg_sync(header)
    }

    fn name(&self) -> &'static str
    {
        "ID3v2.3 Dissector"
    }
}

/// Dissect an ID3v2.3 file from the beginning with specific options
pub fn dissect_id3v2_3_file_with_options(file: &mut File, options: &DebugOptions) -> Result<(), Box<dyn std::error::Error>>
{
    // Read and parse ID3v2 header
    if let Some((major, minor, flags, size)) = read_id3v2_header(file)?
    {
        if major == 3
        {
            if options.show_header == true
            {
                println!("\nID3v2 Header Found:");
                println!("  Version: 2.{}.{}", major, minor);
                println!("  Flags: 0x{:02X}", flags);

                // Interpret header flags
                if flags != 0
                {
                    print!("    ");
                    let mut flag_parts = Vec::new();
                    if flags & 0x80 != 0
                    {
                        flag_parts.push("unsynchronisation");
                    }
                    if flags & 0x40 != 0
                    {
                        flag_parts.push("extended_header");
                    }
                    if flags & 0x20 != 0
                    {
                        flag_parts.push("experimental");
                    }
                    if flag_parts.is_empty() == false
                    {
                        println!("Active: {}", flag_parts.join(", "));
                    }
                }

                println!("  Tag Size: {} bytes", size);

                if size > 100_000_000
                {
                    println!("  WARNING: Extremely large tag size (> 100MB), verify file integrity");
                }
                else if size > 50_000_000
                {
                    println!("  WARNING: Tag size is very large (> 50MB), likely rich podcast with chapter images");
                }
                else if size > 10_000_000
                {
                    println!("  INFO: Large tag size (> 10MB), possibly podcast with embedded chapter content");
                }
            }

            if size > 0
            {
                // Allow very large tags for podcast content with chapter images
                dissect_id3v2_3_with_options(file, size, flags, options)?;
            }
        }
        else if options.show_header == true
        {
            println!("  Expected ID3v2.3, found version 2.{}", major);
        }
    }
    else if options.show_header
    {
        println!("No ID3v2 header found");
    }

    Ok(())
}

pub fn dissect_id3v2_3_with_options(file: &mut File, tag_size: u32, flags: u8, options: &DebugOptions) -> Result<(), Box<dyn std::error::Error>>
{
    if options.show_data == false
    {
        // If not showing data, skip the tag data entirely
        let mut buffer = vec![0u8; tag_size as usize];
        match file.read_exact(&mut buffer)
        {
            | Ok(_) =>
            {
                // Successfully skipped tag data
            }
            | Err(e) =>
            {
                println!("{}", format!("ERROR: Failed to skip tag data: {}", e).bright_red());
                return Err(Box::new(e));
            }
        }
        return Ok(());
    }

    // Diagnostic output
    println!("\nDissecting ID3v2.3 tag (size: {} bytes, flags: 0x{:02X})...", tag_size, flags);

    let mut buffer = vec![0u8; tag_size as usize];
    match file.read_exact(&mut buffer)
    {
        | Ok(_) =>
        {
            println!("Successfully read {} bytes of tag data", tag_size);
        }
        | Err(e) =>
        {
            println!("{}", format!("ERROR: Failed to read tag data: {}", e).bright_red());
            return Err(Box::new(e));
        }
    }

    // Handle unsynchronization if flag is set
    let unsync_flag = flags & 0x80 != 0; // Bit 7
    if unsync_flag
    {
        println!("  Unsynchronization detected - removing sync bytes");
        buffer = remove_unsynchronization(&buffer);
        println!("  After unsynchronization removal: {} bytes", buffer.len());
    }

    println!("\nID3v2.3 Frames:");

    // Check for extended header
    let mut frame_start = 0;
    if flags & 0x40 != 0
    {
        // Extended header flag
        println!("Extended header flag set, parsing...");

        if buffer.len() >= 4
        {
            // ID3v2.3 uses regular big-endian integer for extended header size
            let extended_size = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
            frame_start = 4 + extended_size as usize;

            println!("  Extended header size: {} bytes", extended_size);
            println!("  Frame data starts at offset: {}", frame_start);

            if frame_start > buffer.len()
            {
                println!("  {}", "ERROR: Extended header size exceeds buffer length".bright_red());
                return Err("Invalid extended header size".into());
            }
        }
        else
        {
            println!("  {}", "ERROR: Buffer too small to read extended header size".bright_red());
            return Err("Buffer too small for extended header".into());
        }
    }

    let mut pos = frame_start;

    while pos + 10 <= buffer.len()
    {
        // ID3v2.3 frame header: 4 bytes ID + 4 bytes size + 2 bytes flags
        let frame_id_bytes = &buffer[pos..pos + 4];
        let frame_id = std::str::from_utf8(frame_id_bytes).unwrap_or("????");

        // Stop if we hit padding (null bytes)
        if frame_id.starts_with('\0') || !frame_id.chars().all(|c| c.is_ascii_alphanumeric())
        {
            println!("  Reached padding or end of frames at position 0x{:08X}", pos);
            break;
        }

        // ID3v2.3 uses regular big-endian integers (not synchsafe)
        let frame_size = u32::from_be_bytes([buffer[pos + 4], buffer[pos + 5], buffer[pos + 6], buffer[pos + 7]]);
        let frame_flags = u16::from_be_bytes([buffer[pos + 8], buffer[pos + 9]]);

        // Check if this is a valid ID3v2.3 frame ID
        if is_valid_frame_for_version(frame_id, 3) == false
        {
            // Create a temporary frame for header display even though it's invalid
            let temp_frame = crate::id3v2::frame::Id3v2Frame::new_with_offset(frame_id.to_string(), frame_size, frame_flags, pos, Vec::new());

            // Use the unified frame header display function
            crate::id3v2::tools::display_frame_header(&mut std::io::stdout(), &temp_frame, "    ")?;

            println!("    {}", format!("ERROR: '{}' is not a valid ID3v2.3 frame ID (may be from ID3v2.4 or other version)", frame_id).red());
            println!();

            // Skip the entire frame (header + data) instead of just 1 byte
            if frame_size > 0 && frame_size <= (buffer.len() - pos - 10) as u32
            {
                pos += 10 + frame_size as usize;
            }
            else
            {
                println!("    {}", format!("ERROR: Invalid frame size {}, falling back to 1-byte skip", frame_size).bright_red());
                pos += 1;
            }
            continue;
        }

        // Sanity check frame size
        if frame_size == 0
        {
            println!("  Frame '{}' has zero size, skipping", frame_id);
            pos += 10;
            continue;
        }

        if frame_size > (buffer.len() - pos - 10) as u32
        {
            println!("  Frame '{}' size ({} bytes) exceeds remaining buffer, stopping", frame_id, frame_size);
            break;
        }

        // Create a temporary frame for header display (before full parsing)
        let temp_frame = crate::id3v2::frame::Id3v2Frame::new_with_offset(
            frame_id.to_string(),
            frame_size,
            frame_flags,
            pos,
            Vec::new() // Empty data for header display only
        );

        // Use the unified frame header display function
        crate::id3v2::tools::display_frame_header(&mut std::io::stdout(), &temp_frame, "    ")?;

        // Parse the frame using the new typed system
        match parse_id3v2_3_frame(&buffer, pos)
        {
            | Some(frame) =>
            {
                // Display frame content differently based on dump flag
                if options.show_dump == true
                {
                    // For frames with embedded content, handle display manually
                    if let Some(content) = &frame.content
                    {
                        match content
                        {
                            | crate::id3v2::frame::Id3v2FrameContent::Chapter(chapter_frame) =>
                            {
                                // Display chapter info
                                println!("    Element ID: \"{}\"", chapter_frame.element_id);
                                let start_formatted = crate::id3v2::frames::chapter::format_timestamp(chapter_frame.start_time);
                                let end_formatted = crate::id3v2::frames::chapter::format_timestamp(chapter_frame.end_time);
                                let duration_formatted = crate::id3v2::frames::chapter::format_timestamp(chapter_frame.duration());
                                println!("    Time: {} - {} (duration: {})", start_formatted, end_formatted, duration_formatted);
                                if chapter_frame.has_byte_offsets() == true
                                {
                                    println!("    Byte offsets: {} - {}", chapter_frame.start_offset, chapter_frame.end_offset);
                                }

                                // Show main frame raw data first
                                println!("    Raw data:");
                                let hexdump = crate::hexdump::format_hexdump(&frame.data, 0);
                                for line in hexdump.lines()
                                {
                                    println!("    {}", line);
                                }
                                println!();

                                if chapter_frame.sub_frames.is_empty() == false
                                {
                                    println!("    Sub-frames: {} embedded frame(s)", chapter_frame.sub_frames.len());
                                    println!();

                                    for sub_frame in &chapter_frame.sub_frames
                                    {
                                        // Display embedded frame with hexdump (includes trailing newline)
                                        print!("{}", crate::id3v2::frames::chapter::display_embedded_frame_with_dump(sub_frame, "        "));
                                    }
                                }
                            }
                            | crate::id3v2::frame::Id3v2FrameContent::TableOfContents(toc_frame) =>
                            {
                                // Display TOC info
                                println!("    Element ID: \"{}\"", toc_frame.element_id);
                                if toc_frame.top_level == true
                                {
                                    println!("    Flags: Top-level TOC");
                                }
                                if toc_frame.ordered == true
                                {
                                    println!("    Flags: Ordered");
                                }

                                if toc_frame.child_element_ids.is_empty() == false
                                {
                                    print!("    Child elements ({}): ", toc_frame.child_element_ids.len());
                                    for (i, child_id) in toc_frame.child_element_ids.iter().enumerate()
                                    {
                                        print!("[{}] \"{}\"", i + 1, child_id);
                                        if i < toc_frame.child_element_ids.len() - 1
                                        {
                                            print!(", ");
                                        }
                                    }
                                    println!();
                                }

                                // Show main frame raw data first
                                println!("    Raw data:");
                                let hexdump = crate::hexdump::format_hexdump(&frame.data, 0);
                                for line in hexdump.lines()
                                {
                                    println!("    {}", line);
                                }
                                println!();

                                if toc_frame.sub_frames.is_empty() == false
                                {
                                    println!("    Sub-frames: {} embedded frame(s)", toc_frame.sub_frames.len());
                                    println!();

                                    for sub_frame in &toc_frame.sub_frames
                                    {
                                        // Display embedded frame with hexdump (includes trailing newline)
                                        print!("{}", crate::id3v2::frames::chapter::display_embedded_frame_with_dump(sub_frame, "        "));
                                    }
                                }
                            }
                            | _ =>
                            {
                                // Standard frame display
                                print!("    {}", frame);

                                println!("    Raw data:");
                                // Limit hexdump for APIC frames (cover art) to 128 bytes
                                let hexdump = if frame.id == "APIC"
                                {
                                    crate::hexdump::format_hexdump_limited(&frame.data, 0, Some(128))
                                }
                                else
                                {
                                    crate::hexdump::format_hexdump(&frame.data, 0)
                                };
                                for line in hexdump.lines()
                                {
                                    println!("    {}", line);
                                }
                                println!();
                            }
                        }
                    }
                    else
                    {
                        // No parsed content, show standard display
                        print!("    {}", frame);

                        println!("    Raw data:");
                        // Limit hexdump for APIC frames (cover art) to 128 bytes
                        let hexdump = if frame.id == "APIC"
                        {
                            crate::hexdump::format_hexdump_limited(&frame.data, 0, Some(128))
                        }
                        else
                        {
                            crate::hexdump::format_hexdump(&frame.data, 0)
                        };
                        for line in hexdump.lines()
                        {
                            println!("    {}", line);
                        }
                        println!();
                    }
                }
                else
                {
                    // No dump flag, use standard Display
                    print!("    {}", frame);
                }
            }
            | None =>
            {
                println!("        WARNING: Failed to parse frame, showing raw info");

                let preview_len = std::cmp::min(20, frame_size as usize);
                let preview_data = &buffer[pos + 10..pos + 10 + preview_len];
                print!("          Raw data preview: ");
                for byte in preview_data
                {
                    print!("{:02X} ", byte);
                }
                println!();
            }
        }

        // Move to next frame
        pos += 10 + frame_size as usize;
    }

    Ok(())
}
