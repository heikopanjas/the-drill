/// Format data as a hexdump
pub fn format_hexdump(data: &[u8], base_offset: usize) -> String
{
    format_hexdump_limited(data, base_offset, None)
}

/// Format data as a hexdump with optional byte limit
/// If max_bytes is Some(n), only format first n bytes and append a truncation notice
pub fn format_hexdump_limited(data: &[u8], base_offset: usize, max_bytes: Option<usize>) -> String
{
    let mut output = String::new();

    let (data_to_dump, is_truncated) = match max_bytes
    {
        | Some(limit) if data.len() > limit => (&data[..limit], true),
        | _ => (data, false)
    };

    for (i, chunk) in data_to_dump.chunks(16).enumerate()
    {
        let offset = base_offset + (i * 16);

        // Offset column
        output.push_str(&format!("{:08X}  ", offset));

        // Hex bytes (16 bytes per line, split in groups of 8)
        for (j, byte) in chunk.iter().enumerate()
        {
            if j == 8
            {
                output.push(' ');
            }
            output.push_str(&format!("{:02X} ", byte));
        }

        // Pad if less than 16 bytes
        if chunk.len() < 16
        {
            let padding_count = 16 - chunk.len();
            for j in 0..padding_count
            {
                if chunk.len() + j == 8
                {
                    output.push(' ');
                }
                output.push_str("   ");
            }
        }

        // ASCII representation
        output.push_str(" |");
        for byte in chunk
        {
            if *byte >= 0x20 && *byte <= 0x7E
            {
                output.push(*byte as char);
            }
            else
            {
                output.push('.');
            }
        }
        output.push('|');
        output.push('\n');
    }

    if is_truncated
    {
        output.push_str("<truncated>\n");
    }

    output
}
