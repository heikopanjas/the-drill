use std::fmt;

/// Edit List Box (elst)
#[derive(Debug, Clone)]
pub struct EditListBox
{
    pub version:     u8,
    pub entry_count: u32
}

impl EditListBox
{
    /// Parse elst (Edit List) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("elst box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(EditListBox { version, entry_count })
    }
}

impl fmt::Display for EditListBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Entry Count: {} edit list entries", self.entry_count)?;
        Ok(())
    }
}
