use std::fmt;

/// Media Header Box (mdhd)
#[derive(Debug, Clone)]
pub struct MediaHeaderBox
{
    pub version:           u8,
    pub creation_time:     u64,
    pub modification_time: u64,
    pub timescale:         u32,
    pub duration:          u64,
    pub language:          String
}

impl MediaHeaderBox
{
    /// Parse mdhd (Media Header) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 4
        {
            return Err("mdhd box too short".to_string());
        }

        let version = data[0];

        let (creation_time, modification_time, timescale, duration, lang_offset) = if version == 1
        {
            // 64-bit version
            if data.len() < 36
            {
                return Err("mdhd version 1 box too short".to_string());
            }

            let creation = u64::from_be_bytes([data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11]]);
            let modification = u64::from_be_bytes([data[12], data[13], data[14], data[15], data[16], data[17], data[18], data[19]]);
            let scale = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
            let dur = u64::from_be_bytes([data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31]]);

            (creation, modification, scale, dur, 32)
        }
        else
        {
            // 32-bit version
            if data.len() < 24
            {
                return Err("mdhd version 0 box too short".to_string());
            }

            let creation = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as u64;
            let modification = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as u64;
            let scale = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);
            let dur = u32::from_be_bytes([data[16], data[17], data[18], data[19]]) as u64;

            (creation, modification, scale, dur, 20)
        };

        // Parse language (ISO 639-2/T language code, 3 x 5 bits)
        if data.len() < lang_offset + 2
        {
            return Err("mdhd box too short for language".to_string());
        }

        let lang_code = u16::from_be_bytes([data[lang_offset], data[lang_offset + 1]]);
        let lang_chars: Vec<char> =
            vec![(((lang_code >> 10) & 0x1F) as u8 + 0x60) as char, (((lang_code >> 5) & 0x1F) as u8 + 0x60) as char, ((lang_code & 0x1F) as u8 + 0x60) as char];
        let language = lang_chars.into_iter().collect();

        Ok(MediaHeaderBox { version, creation_time, modification_time, timescale, duration, language })
    }
}

impl fmt::Display for MediaHeaderBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Creation Time: {} (Mac epoch)", self.creation_time)?;
        writeln!(f, "Modification Time: {} (Mac epoch)", self.modification_time)?;
        writeln!(f, "Timescale: {} units/second", self.timescale)?;
        writeln!(f, "Duration: {} units ({:.2} seconds)", self.duration, (self.duration as f64) / (self.timescale as f64))?;
        writeln!(f, "Language: {}", self.language)?;
        Ok(())
    }
}
