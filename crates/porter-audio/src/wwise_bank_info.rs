use std::collections::HashMap;

use crate::AudioError;

/// Parser for wwise sound bank info xml files.
pub struct WwiseBankInfo {
    source_names: HashMap<u32, String>,
}

impl WwiseBankInfo {
    /// Parse wwise sound bank info data.
    pub fn parse(buffer: String) -> Result<Self, AudioError> {
        let mut buffer_data = &buffer[..];
        let mut source_names: HashMap<u32, String> = HashMap::new();

        while !buffer_data.is_empty() {
            let Some(tag_start) = buffer_data.find("<File Id=\"") else {
                return Ok(Self { source_names });
            };

            let tag_number_end = buffer_data[tag_start + 10..]
                .find('"')
                .ok_or(AudioError::WwiseParseNumberEnd)?;

            let tag_number: u32 = buffer_data[tag_start + 10..tag_start + 10 + tag_number_end]
                .parse()
                .map_err(|_| AudioError::WwiseParseNumberParse)?;

            let tag_end = buffer_data[tag_start..]
                .find("</File>")
                .ok_or(AudioError::WwiseParseTagEnd)?;

            let tag_contents = &buffer_data[tag_start..tag_start + tag_end];

            if let Some(cache_path_start) = tag_contents.find("<CachePath>")
                && let Some(cache_path_end) = tag_contents[cache_path_start..].find("</CachePath>")
            {
                source_names.insert(
                    tag_number,
                    tag_contents[cache_path_start + 11..cache_path_start + cache_path_end]
                        .replace("&amp;", "&"),
                );
            } else if let Some(path_start) = tag_contents.find("<Path>")
                && let Some(path_end) = tag_contents[path_start..].find("</Path>")
            {
                source_names.insert(
                    tag_number,
                    tag_contents[path_start + 6..path_start + path_end].replace("&amp;", "&"),
                );
            } else if let Some(short_name_start) = tag_contents.find("<ShortName>")
                && let Some(short_name_end) = tag_contents[short_name_start..].find("</ShortName>")
            {
                source_names.insert(
                    tag_number,
                    tag_contents[short_name_start + 11..short_name_start + short_name_end]
                        .replace("&amp;", "&"),
                );
            } else {
                #[cfg(debug_assertions)]
                println!("Found file tag, but no path found: {:?}", tag_contents);
            }

            buffer_data = &buffer_data[tag_start + tag_end..];
        }

        Ok(Self { source_names })
    }

    /// Attempts to find the original name of a wwise source file.
    pub fn find_name(&self, id: u32) -> Option<String> {
        self.source_names.get(&id).cloned()
    }
}
