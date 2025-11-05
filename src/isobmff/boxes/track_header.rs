use std::fmt;

/// Track Header Box (tkhd)
#[derive(Debug, Clone)]
pub struct TrackHeaderBox
{
    pub version:           u8,
    pub flags:             u32,
    pub creation_time:     u64,
    pub modification_time: u64,
    pub track_id:          u32,
    pub duration:          u64,
    pub layer:             i16,
    pub alternate_group:   i16,
    pub volume:            f64,
    pub width:             f64,
    pub height:            f64
}

impl TrackHeaderBox
{
    /// Parse tkhd (Track Header) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 4
        {
            return Err("tkhd box too short".to_string());
        }

        let version = data[0];
        let flags = u32::from_be_bytes([0, data[1], data[2], data[3]]);

        let (creation_time, modification_time, track_id, duration) = if version == 1
        {
            // 64-bit version
            if data.len() < 40
            {
                return Err("tkhd version 1 box too short".to_string());
            }

            let creation = u64::from_be_bytes([data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11]]);
            let modification = u64::from_be_bytes([data[12], data[13], data[14], data[15], data[16], data[17], data[18], data[19]]);
            let id = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
            // 4 bytes reserved at [24..28]
            let dur = u64::from_be_bytes([data[28], data[29], data[30], data[31], data[32], data[33], data[34], data[35]]);

            (creation, modification, id, dur)
        }
        else
        {
            // 32-bit version
            if data.len() < 28
            {
                return Err("tkhd version 0 box too short".to_string());
            }

            let creation = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as u64;
            let modification = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as u64;
            let id = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);
            // 4 bytes reserved at [16..20]
            let dur = u32::from_be_bytes([data[20], data[21], data[22], data[23]]) as u64;

            (creation, modification, id, dur)
        };

        // Parse layer, alternate_group, volume, width, height
        let base_offset = if version == 1
        {
            36
        }
        else
        {
            24
        };

        if data.len() < base_offset + 52
        {
            return Err("tkhd box too short for additional fields".to_string());
        }

        // 8 bytes reserved at base_offset
        let layer = i16::from_be_bytes([data[base_offset + 8], data[base_offset + 9]]);
        let alternate_group = i16::from_be_bytes([data[base_offset + 10], data[base_offset + 11]]);
        let volume_fixed = i16::from_be_bytes([data[base_offset + 12], data[base_offset + 13]]);
        let volume = (volume_fixed as f64) / 256.0;
        // 2 bytes reserved at base_offset + 14
        // 36 bytes transformation matrix at base_offset + 16

        let width_fixed = u32::from_be_bytes([data[base_offset + 52], data[base_offset + 53], data[base_offset + 54], data[base_offset + 55]]);
        let width = (width_fixed as f64) / 65536.0;

        let height_fixed = u32::from_be_bytes([data[base_offset + 56], data[base_offset + 57], data[base_offset + 58], data[base_offset + 59]]);
        let height = (height_fixed as f64) / 65536.0;

        Ok(TrackHeaderBox { version, flags, creation_time, modification_time, track_id, duration, layer, alternate_group, volume, width, height })
    }
}

impl fmt::Display for TrackHeaderBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(
            f,
            "Flags: 0x{:06X} (Track enabled: {}, In movie: {}, In preview: {})",
            self.flags,
            (self.flags & 0x01) != 0,
            (self.flags & 0x02) != 0,
            (self.flags & 0x04) != 0
        )?;
        writeln!(f, "Creation Time: {} (Mac epoch)", self.creation_time)?;
        writeln!(f, "Modification Time: {} (Mac epoch)", self.modification_time)?;
        writeln!(f, "Track ID: {}", self.track_id)?;
        writeln!(f, "Duration: {} units", self.duration)?;
        writeln!(f, "Layer: {}", self.layer)?;
        writeln!(f, "Alternate Group: {}", self.alternate_group)?;
        writeln!(f, "Volume: {:.2}", self.volume)?;
        writeln!(f, "Width: {:.2} pixels", self.width)?;
        writeln!(f, "Height: {:.2} pixels", self.height)?;
        Ok(())
    }
}
