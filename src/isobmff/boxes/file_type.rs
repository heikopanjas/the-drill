use std::fmt;

/// File Type Box (ftyp)
#[derive(Debug, Clone)]
pub struct FileTypeBox
{
    pub major_brand:       String,
    pub minor_version:     u32,
    pub compatible_brands: Vec<String>
}

impl FileTypeBox
{
    /// Parse ftyp (File Type) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("ftyp box too short".to_string());
        }

        let major_brand = String::from_utf8_lossy(&data[0..4]).to_string();
        let minor_version = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        let mut compatible_brands = Vec::new();
        for chunk in data[8..].chunks(4)
        {
            if chunk.len() == 4
            {
                compatible_brands.push(String::from_utf8_lossy(chunk).to_string());
            }
        }

        Ok(FileTypeBox { major_brand, minor_version, compatible_brands })
    }
}

impl fmt::Display for FileTypeBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Major Brand: '{}'", self.major_brand)?;
        writeln!(f, "Minor Version: {}", self.minor_version)?;
        if self.compatible_brands.is_empty() == false
        {
            write!(f, "Compatible Brands: ")?;
            let brands: Vec<String> = self.compatible_brands.iter().map(|b| format!("'{}'", b)).collect();
            writeln!(f, "{}", brands.join(", "))?;
        }
        Ok(())
    }
}
