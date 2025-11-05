use std::fmt;

/// Chapter Track Reference Box (chap)
#[derive(Debug, Clone)]
pub struct ChapterBox
{
    pub track_ids: Vec<u32>
}

impl ChapterBox
{
    /// Parse chap (Chapter Track Reference) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 4
        {
            return Err("chap box too short".to_string());
        }

        let mut track_ids = Vec::new();
        for chunk in data.chunks(4)
        {
            if chunk.len() == 4
            {
                track_ids.push(u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
            }
        }

        Ok(ChapterBox { track_ids })
    }
}

impl fmt::Display for ChapterBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Chapter Track IDs: {:?}", self.track_ids)?;
        Ok(())
    }
}
