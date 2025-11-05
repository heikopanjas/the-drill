use std::fmt;

/// Movie Header Box (mvhd)
#[derive(Debug, Clone)]
pub struct MovieHeaderBox
{
    pub version:           u8,
    pub creation_time:     u64,
    pub modification_time: u64,
    pub timescale:         u32,
    pub duration:          u64,
    pub rate:              f64,
    pub volume:            f64
}

impl MovieHeaderBox
{
    /// Parse mvhd (Movie Header) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 4
        {
            return Err("mvhd box too short".to_string());
        }

        let version = data[0];
        // flags are at data[1..4] but we don't need them for mvhd

        let (creation_time, modification_time, timescale, duration) = if version == 1
        {
            // 64-bit version
            if data.len() < 36
            {
                return Err("mvhd version 1 box too short".to_string());
            }

            let creation = u64::from_be_bytes([data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11]]);
            let modification = u64::from_be_bytes([data[12], data[13], data[14], data[15], data[16], data[17], data[18], data[19]]);
            let scale = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
            let dur = u64::from_be_bytes([data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31]]);

            (creation, modification, scale, dur)
        }
        else
        {
            // 32-bit version
            if data.len() < 24
            {
                return Err("mvhd version 0 box too short".to_string());
            }

            let creation = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as u64;
            let modification = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as u64;
            let scale = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);
            let dur = u32::from_be_bytes([data[16], data[17], data[18], data[19]]) as u64;

            (creation, modification, scale, dur)
        };

        // Parse rate and volume (same offset for both versions, after the variable-size fields)
        let rate_offset = if version == 1
        {
            32
        }
        else
        {
            20
        };

        if data.len() < rate_offset + 8
        {
            return Err("mvhd box too short for rate/volume".to_string());
        }

        let rate_fixed = i32::from_be_bytes([data[rate_offset], data[rate_offset + 1], data[rate_offset + 2], data[rate_offset + 3]]);
        let rate = (rate_fixed as f64) / 65536.0;

        let volume_fixed = i16::from_be_bytes([data[rate_offset + 4], data[rate_offset + 5]]);
        let volume = (volume_fixed as f64) / 256.0;

        Ok(MovieHeaderBox { version, creation_time, modification_time, timescale, duration, rate, volume })
    }
}

impl fmt::Display for MovieHeaderBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Creation Time: {} (Mac epoch)", self.creation_time)?;
        writeln!(f, "Modification Time: {} (Mac epoch)", self.modification_time)?;
        writeln!(f, "Timescale: {} units/second", self.timescale)?;
        writeln!(f, "Duration: {} units ({:.2} seconds)", self.duration, (self.duration as f64) / (self.timescale as f64))?;
        writeln!(f, "Preferred Rate: {:.2}", self.rate)?;
        writeln!(f, "Preferred Volume: {:.2}", self.volume)?;
        Ok(())
    }
}
