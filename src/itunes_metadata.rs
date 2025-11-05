use std::fmt;

/// iTunes metadata data type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItunesDataType
{
    Implicit,    // 0x00
    Utf8,        // 0x01
    Utf16Be,     // 0x02
    Jpeg,        // 0x0D
    Png,         // 0x0E
    SignedInt,   // 0x15
    UnsignedInt, // 0x16
    Binary(u8)   // Other values
}

impl ItunesDataType
{
    pub fn from_flags(flags: u32) -> Self
    {
        // Data type is in the last byte of flags
        let type_byte = (flags & 0xFF) as u8;

        match type_byte
        {
            | 0x00 => ItunesDataType::Implicit,
            | 0x01 => ItunesDataType::Utf8,
            | 0x02 => ItunesDataType::Utf16Be,
            | 0x0D => ItunesDataType::Jpeg,
            | 0x0E => ItunesDataType::Png,
            | 0x15 => ItunesDataType::SignedInt,
            | 0x16 => ItunesDataType::UnsignedInt,
            | other => ItunesDataType::Binary(other)
        }
    }
}

impl fmt::Display for ItunesDataType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            | ItunesDataType::Implicit => write!(f, "Implicit"),
            | ItunesDataType::Utf8 => write!(f, "UTF-8"),
            | ItunesDataType::Utf16Be => write!(f, "UTF-16 BE"),
            | ItunesDataType::Jpeg => write!(f, "JPEG Image"),
            | ItunesDataType::Png => write!(f, "PNG Image"),
            | ItunesDataType::SignedInt => write!(f, "Signed Integer"),
            | ItunesDataType::UnsignedInt => write!(f, "Unsigned Integer"),
            | ItunesDataType::Binary(type_byte) => write!(f, "Binary (0x{:02X})", type_byte)
        }
    }
}

/// iTunes metadata content
#[derive(Debug, Clone)]
pub enum ItunesContent
{
    Text(String),
    Integer(i64),
    UnsignedInteger(u64),
    Image
    {
        format:    String,
        data_size: usize
    },
    Binary(Vec<u8>),
    TrackNumber
    {
        track:        u16,
        total_tracks: u16
    },
    DiskNumber
    {
        disk:        u16,
        total_disks: u16
    }
}

/// Parsed iTunes metadata box
#[derive(Debug, Clone)]
pub struct ItunesMetadata
{
    pub data_type: ItunesDataType,
    pub content:   ItunesContent
}

impl ItunesMetadata
{
    /// Parse iTunes metadata from a 'data' box
    pub fn parse(box_type: &str, data: &[u8]) -> Result<Self, String>
    {
        // iTunes data box structure:
        // - Version (1 byte)
        // - Flags (3 bytes) - data type indicator
        // - Reserved (4 bytes)
        // - Data (remaining bytes)

        if data.len() < 8
        {
            return Err("iTunes data box too short".to_string());
        }

        let _version = data[0];
        let flags = u32::from_be_bytes([0, data[1], data[2], data[3]]);
        // Skip reserved bytes at [4..8]

        let data_type = ItunesDataType::from_flags(flags);
        let payload = &data[8..];

        let content = match data_type
        {
            | ItunesDataType::Implicit =>
            {
                // Special handling for track and disk numbers with implicit type
                if (box_type == "trkn" || box_type == "disk") && payload.len() >= 6
                {
                    let number = u16::from_be_bytes([payload[2], payload[3]]);
                    let total = u16::from_be_bytes([payload[4], payload[5]]);

                    if box_type == "trkn"
                    {
                        return Ok(ItunesMetadata { data_type, content: ItunesContent::TrackNumber { track: number, total_tracks: total } });
                    }
                    else
                    {
                        return Ok(ItunesMetadata { data_type, content: ItunesContent::DiskNumber { disk: number, total_disks: total } });
                    }
                }
                else
                {
                    // Fall back to text for other implicit types
                    let text = String::from_utf8_lossy(payload).to_string();
                    ItunesContent::Text(text)
                }
            }
            | ItunesDataType::Utf8 =>
            {
                let text = String::from_utf8_lossy(payload).to_string();
                ItunesContent::Text(text)
            }
            | ItunesDataType::Utf16Be =>
            {
                // Decode UTF-16 BE
                let utf16_data: Vec<u16> = payload.chunks_exact(2).map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]])).collect();
                let text = String::from_utf16_lossy(&utf16_data);
                ItunesContent::Text(text)
            }
            | ItunesDataType::SignedInt =>
            {
                let value = match payload.len()
                {
                    | 1 => i8::from_be_bytes([payload[0]]) as i64,
                    | 2 => i16::from_be_bytes([payload[0], payload[1]]) as i64,
                    | 4 => i32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]) as i64,
                    | 8 => i64::from_be_bytes([payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6], payload[7]]),
                    | _ => return Err(format!("Invalid signed integer size: {} bytes", payload.len()))
                };
                ItunesContent::Integer(value)
            }
            | ItunesDataType::UnsignedInt =>
            {
                // Special handling for track and disk numbers
                if box_type == "trkn" || box_type == "disk"
                {
                    if payload.len() >= 6
                    {
                        let number = u16::from_be_bytes([payload[2], payload[3]]);
                        let total = u16::from_be_bytes([payload[4], payload[5]]);

                        if box_type == "trkn"
                        {
                            return Ok(ItunesMetadata { data_type, content: ItunesContent::TrackNumber { track: number, total_tracks: total } });
                        }
                        else
                        {
                            return Ok(ItunesMetadata { data_type, content: ItunesContent::DiskNumber { disk: number, total_disks: total } });
                        }
                    }
                }

                let value = match payload.len()
                {
                    | 1 => payload[0] as u64,
                    | 2 => u16::from_be_bytes([payload[0], payload[1]]) as u64,
                    | 4 => u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]) as u64,
                    | 8 => u64::from_be_bytes([payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6], payload[7]]),
                    | _ => return Err(format!("Invalid unsigned integer size: {} bytes", payload.len()))
                };
                ItunesContent::UnsignedInteger(value)
            }
            | ItunesDataType::Jpeg => ItunesContent::Image { format: "JPEG".to_string(), data_size: payload.len() },
            | ItunesDataType::Png => ItunesContent::Image { format: "PNG".to_string(), data_size: payload.len() },
            | ItunesDataType::Binary(_) => ItunesContent::Binary(payload.to_vec())
        };

        Ok(ItunesMetadata { data_type, content })
    }
}

impl fmt::Display for ItunesMetadata
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Data Type: {}", self.data_type)?;

        match &self.content
        {
            | ItunesContent::Text(text) => writeln!(f, "Value: \"{}\"", text)?,
            | ItunesContent::Integer(value) => writeln!(f, "Value: {}", value)?,
            | ItunesContent::UnsignedInteger(value) => writeln!(f, "Value: {}", value)?,
            | ItunesContent::Image { format, data_size } => writeln!(f, "Value: {} image, {} bytes", format, data_size)?,
            | ItunesContent::Binary(data) => writeln!(f, "Value: Binary data, {} bytes", data.len())?,
            | ItunesContent::TrackNumber { track, total_tracks } =>
            {
                if *total_tracks > 0
                {
                    writeln!(f, "Value: Track {} of {}", track, total_tracks)?
                }
                else
                {
                    writeln!(f, "Value: Track {}", track)?
                }
            }
            | ItunesContent::DiskNumber { disk, total_disks } =>
            {
                if *total_disks > 0
                {
                    writeln!(f, "Value: Disk {} of {}", disk, total_disks)?
                }
                else
                {
                    writeln!(f, "Value: Disk {}", disk)?
                }
            }
        }

        Ok(())
    }
}
