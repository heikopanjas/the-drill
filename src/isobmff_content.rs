use std::fmt;

/// Parsed ISOBMFF box content for various box types
#[derive(Debug, Clone)]
pub enum IsobmffContent
{
    FileType
    {
        major_brand: String, minor_version: u32, compatible_brands: Vec<String>
    },
    MovieHeader
    {
        version:           u8,
        creation_time:     u64,
        modification_time: u64,
        timescale:         u32,
        duration:          u64,
        rate:              f64,
        volume:            f64
    },
    TrackHeader
    {
        version:           u8,
        flags:             u32,
        creation_time:     u64,
        modification_time: u64,
        track_id:          u32,
        duration:          u64,
        layer:             i16,
        alternate_group:   i16,
        volume:            f64,
        width:             f64,
        height:            f64
    },
    MediaHeader
    {
        version:           u8,
        creation_time:     u64,
        modification_time: u64,
        timescale:         u32,
        duration:          u64,
        language:          String
    },
    HandlerReference
    {
        version: u8, handler_type: String, manufacturer: String, name: String
    },
    VideoMediaHeader
    {
        version: u8, graphics_mode: u16, opcolor: [u16; 3]
    },
    SoundMediaHeader
    {
        version: u8, balance: f64
    },
    DataReference
    {
        version: u8, entry_count: u32
    },
    SampleDescription
    {
        version: u8, entry_count: u32, entries: Vec<String>
    },
    TimeToSample
    {
        version: u8, entry_count: u32
    },
    SampleToChunk
    {
        version: u8, entry_count: u32
    },
    SampleSize
    {
        version: u8, sample_size: u32, sample_count: u32
    },
    ChunkOffset
    {
        version: u8, entry_count: u32
    },
    ChunkOffset64
    {
        version: u8, entry_count: u32
    },
    EditList
    {
        version: u8, entry_count: u32
    },
    UrlEntry
    {
        version: u8, flags: u32, location: String
    },
    UrnEntry
    {
        version: u8, flags: u32, name: String, location: String
    },
    ChapterReference
    {
        track_ids: Vec<u32>
    },
    NullMediaHeader
    {
        version: u8
    },
    MetadataMean
    {
        version: u8, namespace: String
    },
    MetadataName
    {
        version: u8, name: String
    }
}

impl IsobmffContent
{
    /// Parse ftyp (File Type) box
    pub fn parse_ftyp(data: &[u8]) -> Result<Self, String>
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

        Ok(IsobmffContent::FileType { major_brand, minor_version, compatible_brands })
    }

    /// Parse mvhd (Movie Header) box
    pub fn parse_mvhd(data: &[u8]) -> Result<Self, String>
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

        Ok(IsobmffContent::MovieHeader { version, creation_time, modification_time, timescale, duration, rate, volume })
    }

    /// Parse tkhd (Track Header) box
    pub fn parse_tkhd(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 4
        {
            return Err("tkhd box too short".to_string());
        }

        let version = data[0];
        let flags = u32::from_be_bytes([0, data[1], data[2], data[3]]);

        let (creation_time, modification_time, track_id, duration) = if version == 1
        {
            // 64-bit version
            if data.len() < 40
            {
                return Err("tkhd version 1 box too short".to_string());
            }

            let creation = u64::from_be_bytes([data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11]]);
            let modification = u64::from_be_bytes([data[12], data[13], data[14], data[15], data[16], data[17], data[18], data[19]]);
            let id = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
            // 4 bytes reserved at [24..28]
            let dur = u64::from_be_bytes([data[28], data[29], data[30], data[31], data[32], data[33], data[34], data[35]]);

            (creation, modification, id, dur)
        }
        else
        {
            // 32-bit version
            if data.len() < 28
            {
                return Err("tkhd version 0 box too short".to_string());
            }

            let creation = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as u64;
            let modification = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as u64;
            let id = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);
            // 4 bytes reserved at [16..20]
            let dur = u32::from_be_bytes([data[20], data[21], data[22], data[23]]) as u64;

            (creation, modification, id, dur)
        };

        // Parse layer, alternate_group, volume, width, height
        let base_offset = if version == 1
        {
            36
        }
        else
        {
            24
        };

        if data.len() < base_offset + 52
        {
            return Err("tkhd box too short for additional fields".to_string());
        }

        // 8 bytes reserved at base_offset
        let layer = i16::from_be_bytes([data[base_offset + 8], data[base_offset + 9]]);
        let alternate_group = i16::from_be_bytes([data[base_offset + 10], data[base_offset + 11]]);
        let volume_fixed = i16::from_be_bytes([data[base_offset + 12], data[base_offset + 13]]);
        let volume = (volume_fixed as f64) / 256.0;
        // 2 bytes reserved at base_offset + 14
        // 36 bytes transformation matrix at base_offset + 16

        let width_fixed = u32::from_be_bytes([data[base_offset + 52], data[base_offset + 53], data[base_offset + 54], data[base_offset + 55]]);
        let width = (width_fixed as f64) / 65536.0;

        let height_fixed = u32::from_be_bytes([data[base_offset + 56], data[base_offset + 57], data[base_offset + 58], data[base_offset + 59]]);
        let height = (height_fixed as f64) / 65536.0;

        Ok(IsobmffContent::TrackHeader { version, flags, creation_time, modification_time, track_id, duration, layer, alternate_group, volume, width, height })
    }

    /// Parse mdhd (Media Header) box
    pub fn parse_mdhd(data: &[u8]) -> Result<Self, String>
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

        Ok(IsobmffContent::MediaHeader { version, creation_time, modification_time, timescale, duration, language })
    }

    /// Parse hdlr (Handler Reference) box
    pub fn parse_hdlr(data: &[u8]) -> Result<Self, String>
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

        Ok(IsobmffContent::HandlerReference { version, handler_type, manufacturer, name })
    }

    /// Parse vmhd (Video Media Header) box
    pub fn parse_vmhd(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 12
        {
            return Err("vmhd box too short".to_string());
        }

        let version = data[0];
        let graphics_mode = u16::from_be_bytes([data[4], data[5]]);
        let opcolor = [u16::from_be_bytes([data[6], data[7]]), u16::from_be_bytes([data[8], data[9]]), u16::from_be_bytes([data[10], data[11]])];

        Ok(IsobmffContent::VideoMediaHeader { version, graphics_mode, opcolor })
    }

    /// Parse smhd (Sound Media Header) box
    pub fn parse_smhd(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("smhd box too short".to_string());
        }

        let version = data[0];
        let balance_fixed = i16::from_be_bytes([data[4], data[5]]);
        let balance = (balance_fixed as f64) / 256.0;

        Ok(IsobmffContent::SoundMediaHeader { version, balance })
    }

    /// Parse dref (Data Reference) box
    pub fn parse_dref(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("dref box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(IsobmffContent::DataReference { version, entry_count })
    }

    /// Parse stsd (Sample Description) box
    pub fn parse_stsd(data: &[u8]) -> Result<Self, String>
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

        Ok(IsobmffContent::SampleDescription { version, entry_count, entries })
    }

    /// Parse stts (Time-to-Sample) box
    pub fn parse_stts(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("stts box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(IsobmffContent::TimeToSample { version, entry_count })
    }

    /// Parse stsc (Sample-to-Chunk) box
    pub fn parse_stsc(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("stsc box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(IsobmffContent::SampleToChunk { version, entry_count })
    }

    /// Parse stsz (Sample Size) box
    pub fn parse_stsz(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 12
        {
            return Err("stsz box too short".to_string());
        }

        let version = data[0];
        let sample_size = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let sample_count = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        Ok(IsobmffContent::SampleSize { version, sample_size, sample_count })
    }

    /// Parse stco (Chunk Offset) box
    pub fn parse_stco(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("stco box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(IsobmffContent::ChunkOffset { version, entry_count })
    }

    /// Parse co64 (64-bit Chunk Offset) box
    pub fn parse_co64(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("co64 box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(IsobmffContent::ChunkOffset64 { version, entry_count })
    }

    /// Parse elst (Edit List) box
    pub fn parse_elst(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 8
        {
            return Err("elst box too short".to_string());
        }

        let version = data[0];
        let entry_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        Ok(IsobmffContent::EditList { version, entry_count })
    }

    /// Parse url  (URL entry) box
    pub fn parse_url(data: &[u8]) -> Result<Self, String>
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

        Ok(IsobmffContent::UrlEntry { version, flags, location })
    }

    /// Parse urn  (URN entry) box
    pub fn parse_urn(data: &[u8]) -> Result<Self, String>
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

        Ok(IsobmffContent::UrnEntry { version, flags, name, location })
    }

    /// Parse chap (Chapter Track Reference) box
    pub fn parse_chap(data: &[u8]) -> Result<Self, String>
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

        Ok(IsobmffContent::ChapterReference { track_ids })
    }

    /// Parse nmhd (Null Media Header) box
    pub fn parse_nmhd(data: &[u8]) -> Result<Self, String>
    {
        if data.len() < 4
        {
            return Err("nmhd box too short".to_string());
        }

        let version = data[0];
        Ok(IsobmffContent::NullMediaHeader { version })
    }

    /// Parse mean (iTunes Metadata Mean/Namespace) box
    pub fn parse_mean(data: &[u8]) -> Result<Self, String>
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

        Ok(IsobmffContent::MetadataMean { version, namespace })
    }

    /// Parse name (iTunes Metadata Name) box
    pub fn parse_name(data: &[u8]) -> Result<Self, String>
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

        Ok(IsobmffContent::MetadataName { version, name })
    }
}

impl fmt::Display for IsobmffContent
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            | IsobmffContent::FileType { major_brand, minor_version, compatible_brands } =>
            {
                writeln!(f, "Major Brand: '{}'", major_brand)?;
                writeln!(f, "Minor Version: {}", minor_version)?;
                if !compatible_brands.is_empty()
                {
                    write!(f, "Compatible Brands: ")?;
                    let brands: Vec<String> = compatible_brands.iter().map(|b| format!("'{}'", b)).collect();
                    writeln!(f, "{}", brands.join(", "))?;
                }
            }
            | IsobmffContent::MovieHeader { version, creation_time, modification_time, timescale, duration, rate, volume } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Creation Time: {} (Mac epoch)", creation_time)?;
                writeln!(f, "Modification Time: {} (Mac epoch)", modification_time)?;
                writeln!(f, "Timescale: {} units/second", timescale)?;
                writeln!(f, "Duration: {} units ({:.2} seconds)", duration, (*duration as f64) / (*timescale as f64))?;
                writeln!(f, "Preferred Rate: {:.2}", rate)?;
                writeln!(f, "Preferred Volume: {:.2}", volume)?;
            }
            | IsobmffContent::TrackHeader { version, flags, creation_time, modification_time, track_id, duration, layer, alternate_group, volume, width, height } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(
                    f,
                    "Flags: 0x{:06X} (Track enabled: {}, In movie: {}, In preview: {})",
                    flags,
                    (flags & 0x01) != 0,
                    (flags & 0x02) != 0,
                    (flags & 0x04) != 0
                )?;
                writeln!(f, "Creation Time: {} (Mac epoch)", creation_time)?;
                writeln!(f, "Modification Time: {} (Mac epoch)", modification_time)?;
                writeln!(f, "Track ID: {}", track_id)?;
                writeln!(f, "Duration: {} units", duration)?;
                writeln!(f, "Layer: {}", layer)?;
                writeln!(f, "Alternate Group: {}", alternate_group)?;
                writeln!(f, "Volume: {:.2}", volume)?;
                writeln!(f, "Width: {:.2} pixels", width)?;
                writeln!(f, "Height: {:.2} pixels", height)?;
            }
            | IsobmffContent::MediaHeader { version, creation_time, modification_time, timescale, duration, language } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Creation Time: {} (Mac epoch)", creation_time)?;
                writeln!(f, "Modification Time: {} (Mac epoch)", modification_time)?;
                writeln!(f, "Timescale: {} units/second", timescale)?;
                writeln!(f, "Duration: {} units ({:.2} seconds)", duration, (*duration as f64) / (*timescale as f64))?;
                writeln!(f, "Language: {}", language)?;
            }
            | IsobmffContent::HandlerReference { version, handler_type, manufacturer, name } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Handler Type: '{}' ({})", handler_type, Self::get_handler_name(handler_type))?;
                if !manufacturer.is_empty() && manufacturer.chars().any(|c| c.is_alphanumeric())
                {
                    writeln!(f, "Manufacturer: '{}'", manufacturer)?;
                }
                if !name.is_empty()
                {
                    writeln!(f, "Name: \"{}\"", name)?;
                }
            }
            | IsobmffContent::VideoMediaHeader { version, graphics_mode, opcolor } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Graphics Mode: {}", graphics_mode)?;
                writeln!(f, "OpColor: R={}, G={}, B={}", opcolor[0], opcolor[1], opcolor[2])?;
            }
            | IsobmffContent::SoundMediaHeader { version, balance } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Balance: {:.2} (0=center, -1=full left, 1=full right)", balance)?;
            }
            | IsobmffContent::DataReference { version, entry_count } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Entry Count: {}", entry_count)?;
            }
            | IsobmffContent::SampleDescription { version, entry_count, entries } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Entry Count: {}", entry_count)?;
                if !entries.is_empty()
                {
                    write!(f, "Sample Entries: ")?;
                    let entry_list: Vec<String> = entries.iter().map(|e| format!("'{}'", e)).collect();
                    writeln!(f, "{}", entry_list.join(", "))?;
                }
            }
            | IsobmffContent::TimeToSample { version, entry_count } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Entry Count: {} time-to-sample entries", entry_count)?;
            }
            | IsobmffContent::SampleToChunk { version, entry_count } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Entry Count: {} sample-to-chunk entries", entry_count)?;
            }
            | IsobmffContent::SampleSize { version, sample_size, sample_count } =>
            {
                writeln!(f, "Version: {}", version)?;
                if *sample_size == 0
                {
                    writeln!(f, "Sample Size: Variable")?;
                    writeln!(f, "Sample Count: {} (with individual sizes)", sample_count)?;
                }
                else
                {
                    writeln!(f, "Sample Size: {} bytes (constant)", sample_size)?;
                    writeln!(f, "Sample Count: {}", sample_count)?;
                }
            }
            | IsobmffContent::ChunkOffset { version, entry_count } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Entry Count: {} chunk offsets (32-bit)", entry_count)?;
            }
            | IsobmffContent::ChunkOffset64 { version, entry_count } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Entry Count: {} chunk offsets (64-bit)", entry_count)?;
            }
            | IsobmffContent::EditList { version, entry_count } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Entry Count: {} edit list entries", entry_count)?;
            }
            | IsobmffContent::UrlEntry { version, flags, location } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Flags: 0x{:06X}", flags)?;
                if !location.is_empty()
                {
                    writeln!(f, "Location: {}", location)?;
                }
            }
            | IsobmffContent::UrnEntry { version, flags, name, location } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Flags: 0x{:06X}", flags)?;
                if !name.is_empty()
                {
                    writeln!(f, "Name: {}", name)?;
                }
                if !location.is_empty()
                {
                    writeln!(f, "Location: {}", location)?;
                }
            }
            | IsobmffContent::ChapterReference { track_ids } =>
            {
                writeln!(f, "Chapter Track IDs: {:?}", track_ids)?;
            }
            | IsobmffContent::NullMediaHeader { version } =>
            {
                writeln!(f, "Version: {}", version)?;
            }
            | IsobmffContent::MetadataMean { version, namespace } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Namespace: {}", namespace)?;
            }
            | IsobmffContent::MetadataName { version, name } =>
            {
                writeln!(f, "Version: {}", version)?;
                writeln!(f, "Name: {}", name)?;
            }
        }

        Ok(())
    }
}

impl IsobmffContent
{
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
