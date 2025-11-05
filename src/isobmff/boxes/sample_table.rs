use std::fmt;

/// Sample Description Box (stsd)
#[derive(Debug, Clone)]
pub struct SampleDescriptionBox
{
    pub version:     u8,
    pub entry_count: u32,
    pub entries:     Vec<String>
}

impl SampleDescriptionBox
{
    /// Parse stsd (Sample Description) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("stsd box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        // Try to extract sample entry types (format codes)
        let mut entries = Vec::new();
        let mut offset = 8;

        for _ in 0..entry_count
        {
            if offset + 8 > data.len()
            {
                break;
            }

            let entry_size = u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
            let format = String::from_utf8_lossy(&data[offset + 4..offset + 8]).to_string();
            entries.push(format);

            offset += entry_size as usize;
            if offset >= data.len()
            {
                break;
            }
        }

        Ok(SampleDescriptionBox { version, entry_count, entries })
    }
}

impl fmt::Display for SampleDescriptionBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Entry Count: {}", self.entry_count)?;
        if !self.entries.is_empty()
        {
            write!(f, "Sample Entries: ")?;
            let entry_list: Vec<String> = self.entries.iter().map(|e| format!("'{}'", e)).collect();
            writeln!(f, "{}", entry_list.join(", "))?;
        }
        Ok(())
    }
}

/// Time-to-Sample Box (stts)
#[derive(Debug, Clone)]
pub struct TimeToSampleBox
{
    pub version:     u8,
    pub entry_count: u32
}

impl TimeToSampleBox
{
    /// Parse stts (Time-to-Sample) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("stts box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(TimeToSampleBox { version, entry_count })
    }
}

impl fmt::Display for TimeToSampleBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Entry Count: {} time-to-sample entries", self.entry_count)?;
        Ok(())
    }
}

/// Sample-to-Chunk Box (stsc)
#[derive(Debug, Clone)]
pub struct SampleToChunkBox
{
    pub version:     u8,
    pub entry_count: u32
}

impl SampleToChunkBox
{
    /// Parse stsc (Sample-to-Chunk) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("stsc box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(SampleToChunkBox { version, entry_count })
    }
}

impl fmt::Display for SampleToChunkBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Entry Count: {} sample-to-chunk entries", self.entry_count)?;
        Ok(())
    }
}

/// Sample Size Box (stsz)
#[derive(Debug, Clone)]
pub struct SampleSizeBox
{
    pub version:      u8,
    pub sample_size:  u32,
    pub sample_count: u32
}

impl SampleSizeBox
{
    /// Parse stsz (Sample Size) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 12
        {
            return Err("stsz box too short".to_string());
        }

        let version = data[0];
        let sample_size = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let sample_count = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        Ok(SampleSizeBox { version, sample_size, sample_count })
    }
}

impl fmt::Display for SampleSizeBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        if self.sample_size == 0
        {
            writeln!(f, "Sample Size: Variable")?;
            writeln!(f, "Sample Count: {} (with individual sizes)", self.sample_count)?;
        }
        else
        {
            writeln!(f, "Sample Size: {} bytes (constant)", self.sample_size)?;
            writeln!(f, "Sample Count: {}", self.sample_count)?;
        }
        Ok(())
    }
}

/// Chunk Offset Box (stco)
#[derive(Debug, Clone)]
pub struct ChunkOffsetBox
{
    pub version:     u8,
    pub entry_count: u32
}

impl ChunkOffsetBox
{
    /// Parse stco (Chunk Offset) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("stco box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(ChunkOffsetBox { version, entry_count })
    }
}

impl fmt::Display for ChunkOffsetBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Entry Count: {} chunk offsets (32-bit)", self.entry_count)?;
        Ok(())
    }
}

/// 64-bit Chunk Offset Box (co64)
#[derive(Debug, Clone)]
pub struct ChunkOffset64Box
{
    pub version:     u8,
    pub entry_count: u32
}

impl ChunkOffset64Box
{
    /// Parse co64 (64-bit Chunk Offset) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("co64 box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(ChunkOffset64Box { version, entry_count })
    }
}

impl fmt::Display for ChunkOffset64Box
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Entry Count: {} chunk offsets (64-bit)", self.entry_count)?;
        Ok(())
    }
}
