use std::fmt;

/// iTunes Metadata Mean/Namespace Box (mean)
#[derive(Debug, Clone)]
pub struct MetadataMeanBox
{
    pub version:   u8,
    pub namespace: String
}

impl MetadataMeanBox
{
    /// Parse mean (iTunes Metadata Mean/Namespace) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 4
        {
            return Err("mean box too short".to_string());
        }

        let version = data[0];
        let namespace = if data.len() > 4
        {
            String::from_utf8_lossy(&data[4..]).trim_end_matches('\0').to_string()
        }
        else
        {
            String::new()
        };

        Ok(MetadataMeanBox { version, namespace })
    }
}

impl fmt::Display for MetadataMeanBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Namespace: {}", self.namespace)?;
        Ok(())
    }
}

/// iTunes Metadata Name Box (name)
#[derive(Debug, Clone)]
pub struct MetadataNameBox
{
    pub version: u8,
    pub name:    String
}

impl MetadataNameBox
{
    /// Parse name (iTunes Metadata Name) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 4
        {
            return Err("name box too short".to_string());
        }

        let version = data[0];
        let name = if data.len() > 4
        {
            String::from_utf8_lossy(&data[4..]).trim_end_matches('\0').to_string()
        }
        else
        {
            String::new()
        };

        Ok(MetadataNameBox { version, name })
    }
}

impl fmt::Display for MetadataNameBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Name: {}", self.name)?;
        Ok(())
    }
}
