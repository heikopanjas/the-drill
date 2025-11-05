use std::fmt;

/// Data Reference Box (dref)
#[derive(Debug, Clone)]
pub struct DataReferenceBox
{
    pub version:     u8,
    pub entry_count: u32
}

impl DataReferenceBox
{
    /// Parse dref (Data Reference) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("dref box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(DataReferenceBox { version, entry_count })
    }
}

impl fmt::Display for DataReferenceBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Entry Count: {}", self.entry_count)?;
        Ok(())
    }
}

/// URL Entry Box (url )
#[derive(Debug, Clone)]
pub struct UrlEntryBox
{
    pub version:  u8,
    pub flags:    u32,
    pub location: String
}

impl UrlEntryBox
{
    /// Parse url  (URL entry) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 4
        {
            return Err("url box too short".to_string());
        }

        let version = data[0];
        let flags = u32::from_be_bytes([0, data[1], data[2], data[3]]);

        // If flag 0x000001 is set, data is in the same file (no URL)
        let location = if (flags & 0x000001) != 0
        {
            "(data in same file)".to_string()
        }
        else if data.len() > 4
        {
            String::from_utf8_lossy(&data[4..]).trim_end_matches('\0').to_string()
        }
        else
        {
            String::new()
        };

        Ok(UrlEntryBox { version, flags, location })
    }
}

impl fmt::Display for UrlEntryBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Flags: 0x{:06X}", self.flags)?;
        if !self.location.is_empty()
        {
            writeln!(f, "Location: {}", self.location)?;
        }
        Ok(())
    }
}

/// URN Entry Box (urn )
#[derive(Debug, Clone)]
pub struct UrnEntryBox
{
    pub version:  u8,
    pub flags:    u32,
    pub name:     String,
    pub location: String
}

impl UrnEntryBox
{
    /// Parse urn  (URN entry) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 4
        {
            return Err("urn box too short".to_string());
        }

        let version = data[0];
        let flags = u32::from_be_bytes([0, data[1], data[2], data[3]]);

        // Parse null-terminated name and location strings
        let mut name = String::new();
        let mut location = String::new();

        if data.len() > 4
        {
            let payload = &data[4..];
            if let Some(null_pos) = payload.iter().position(|&b| b == 0)
            {
                name = String::from_utf8_lossy(&payload[..null_pos]).to_string();
                if null_pos + 1 < payload.len()
                {
                    location = String::from_utf8_lossy(&payload[null_pos + 1..]).trim_end_matches('\0').to_string();
                }
            }
        }

        Ok(UrnEntryBox { version, flags, name, location })
    }
}

impl fmt::Display for UrnEntryBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Flags: 0x{:06X}", self.flags)?;
        if !self.name.is_empty()
        {
            writeln!(f, "Name: {}", self.name)?;
        }
        if !self.location.is_empty()
        {
            writeln!(f, "Location: {}", self.location)?;
        }
        Ok(())
    }
}
