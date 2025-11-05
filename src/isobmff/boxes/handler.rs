use std::fmt;

/// Handler Reference Box (hdlr)
#[derive(Debug, Clone)]
pub struct HandlerBox
{
    pub version:      u8,
    pub handler_type: String,
    pub manufacturer: String,
    pub name:         String
}

impl HandlerBox
{
    /// Parse hdlr (Handler Reference) box
    pub fn parse(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 24
        {
            return Err("hdlr box too short".to_string());
        }

        let version = data[0];
        // 4 bytes pre_defined at [4..8]
        let handler_type = String::from_utf8_lossy(&data[8..12]).to_string();
        // 12 bytes reserved at [12..24], first 4 bytes often contain manufacturer code
        let manufacturer = String::from_utf8_lossy(&data[12..16]).trim_end_matches('\0').to_string();

        // Name is null-terminated string at the end
        let name = if data.len() > 24
        {
            let name_data = &data[24..];
            let name_str = String::from_utf8_lossy(name_data);
            // Remove null terminator if present
            name_str.trim_end_matches('\0').to_string()
        }
        else
        {
            String::new()
        };

        Ok(HandlerBox { version, handler_type, manufacturer, name })
    }

    /// Get human-readable handler type name
    fn get_handler_name(handler_type: &str) -> &'static str
    {
        match handler_type
        {
            | "vide" => "Video Track",
            | "soun" => "Audio Track",
            | "hint" => "Hint Track",
            | "meta" => "Metadata Track",
            | "mdir" => "Metadata Directory",
            | "auxv" => "Auxiliary Video Track",
            | "text" => "Text/Subtitle Track",
            | "sbtl" => "Subtitle Track",
            | "subt" => "Subtitle Track",
            | "clcp" => "Closed Caption Track",
            | "tmcd" => "Timecode Track",
            | _ => "Unknown Handler"
        }
    }
}

impl fmt::Display for HandlerBox
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Handler Type: '{}' ({})", self.handler_type, Self::get_handler_name(&self.handler_type))?;
        if !self.manufacturer.is_empty() && self.manufacturer.chars().any(|c| c.is_alphanumeric())
        {
            writeln!(f, "Manufacturer: '{}'", self.manufacturer)?;
        }
        if !self.name.is_empty()
        {
            writeln!(f, "Name: \"{}\"", self.name)?;
        }
        Ok(())
    }
}
