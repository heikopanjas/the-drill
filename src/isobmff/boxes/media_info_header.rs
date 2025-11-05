use std::fmt;

/// Video Media Header Box (vmhd)
#[derive(Debug, Clone)]
pub struct VideoMediaHeaderBox
{
    pub version:       u8,
    pub graphics_mode: u16,
    pub opcolor:       [u16; 3]
}

impl VideoMediaHeaderBox
{
    /// Parse vmhd (Video Media Header) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 12
        {
            return Err("vmhd box too short".to_string());
        }

        let version = data[0];
        let graphics_mode = u16::from_be_bytes([data[4], data[5]]);
        let opcolor = [u16::from_be_bytes([data[6], data[7]]), u16::from_be_bytes([data[8], data[9]]), u16::from_be_bytes([data[10], data[11]])];

        Ok(VideoMediaHeaderBox { version, graphics_mode, opcolor })
    }
}

impl fmt::Display for VideoMediaHeaderBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Graphics Mode: {}", self.graphics_mode)?;
        writeln!(f, "OpColor: R={}, G={}, B={}", self.opcolor[0], self.opcolor[1], self.opcolor[2])?;
        Ok(())
    }
}

/// Sound Media Header Box (smhd)
#[derive(Debug, Clone)]
pub struct SoundMediaHeaderBox
{
    pub version: u8,
    pub balance: f64
}

impl SoundMediaHeaderBox
{
    /// Parse smhd (Sound Media Header) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("smhd box too short".to_string());
        }

        let version = data[0];
        let balance_fixed = i16::from_be_bytes([data[4], data[5]]);
        let balance = (balance_fixed as f64) / 256.0;

        Ok(SoundMediaHeaderBox { version, balance })
    }
}

impl fmt::Display for SoundMediaHeaderBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Balance: {:.2} (0=center, -1=full left, 1=full right)", self.balance)?;
        Ok(())
    }
}

/// Null Media Header Box (nmhd)
#[derive(Debug, Clone)]
pub struct NullMediaHeaderBox
{
    pub version: u8
}

impl NullMediaHeaderBox
{
    /// Parse nmhd (Null Media Header) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 4
        {
            return Err("nmhd box too short".to_string());
        }

        let version = data[0];
        Ok(NullMediaHeaderBox { version })
    }
}

impl fmt::Display for NullMediaHeaderBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        Ok(())
    }
}
