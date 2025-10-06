use std::num::ParseIntError;

/// Ways to filter on a number range.
#[derive(Debug, Clone, Copy)]
struct SearchRange {
    min: u32,
    max: u32,
}

impl Default for SearchRange {
    fn default() -> Self {
        Self {
            min: u32::MIN,
            max: u32::MAX,
        }
    }
}

/// Ways to filter on a search name.
enum SearchName {
    Contained(String),
    NotContained(String),
}

/// The searchable data for an asset.
pub struct SearchAsset {
    bone_count: u32,
    mesh_count: u32,
    frame_count: u32,
    frame_rate: u32,
    width: u32,
    height: u32,
    channels: u32,
    name: String,
}

impl SearchAsset {
    /// Constructs a new search asset with the asset's name.
    pub const fn new(name: String) -> Self {
        Self {
            bone_count: 0,
            mesh_count: 0,
            frame_count: 0,
            frame_rate: 0,
            width: 0,
            height: 0,
            channels: 0,
            name,
        }
    }

    /// Sets the count of bones this asset has.
    pub const fn bone_count(mut self, count: u32) -> Self {
        self.bone_count = count;
        self
    }

    /// Sets the count of meshes this asset has.
    pub const fn mesh_count(mut self, count: u32) -> Self {
        self.mesh_count = count;
        self
    }

    /// Sets the count of frames this asset has.
    pub const fn frame_count(mut self, count: u32) -> Self {
        self.frame_count = count;
        self
    }

    /// Sets the frame rate this asset has.
    pub const fn frame_rate(mut self, rate: u32) -> Self {
        self.frame_rate = rate;
        self
    }

    /// Sets the width this asset has.
    pub const fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Sets the height this asset has.
    pub const fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Sets the channels this asset has.
    pub const fn channels(mut self, channels: u32) -> Self {
        self.channels = channels;
        self
    }
}

/// A compiled search term.
#[repr(align(64))]
pub struct SearchTerm {
    bone_count: SearchRange,
    mesh_count: SearchRange,
    frame_count: SearchRange,
    frame_rate: SearchRange,
    width: SearchRange,
    height: SearchRange,
    channels: SearchRange,
    search_names: [Option<SearchName>; 5],
}

impl SearchTerm {
    /// Compile a search command into a reusable search structure.
    pub fn compile(search: &str) -> Self {
        // Always process search terms as lowercase for case insensitivity.
        let search = search.to_lowercase();

        // Get the commands, up to 5 of them in one search term.
        let commands = search.splitn(5, ',');

        let mut bone_count = SearchRange::default();
        let mut mesh_count = SearchRange::default();
        let mut frame_count = SearchRange::default();
        let mut frame_rate = SearchRange::default();
        let mut width = SearchRange::default();
        let mut height = SearchRange::default();
        let mut channels = SearchRange::default();

        let mut search_names: [Option<SearchName>; 5] = [const { None }; 5];
        let mut search_names_index = 0;

        for command in commands {
            if let Some(command) = command.strip_prefix("bonecount:") {
                let _ = parse_search_number(command, &mut bone_count);
            } else if let Some(command) = command.strip_prefix("meshcount:") {
                let _ = parse_search_number(command, &mut mesh_count);
            } else if let Some(command) = command.strip_prefix("framecount:") {
                let _ = parse_search_number(command, &mut frame_count);
            } else if let Some(command) = command.strip_prefix("framerate:") {
                let _ = parse_search_number(command, &mut frame_rate);
            } else if let Some(command) = command.strip_prefix("width:") {
                let _ = parse_search_number(command, &mut width);
            } else if let Some(command) = command.strip_prefix("height:") {
                let _ = parse_search_number(command, &mut height);
            } else if let Some(command) = command.strip_prefix("channels:") {
                let _ = parse_search_number(command, &mut channels);
            } else if let Some(command) = command.strip_prefix('!') {
                let command = command.trim();

                if !command.is_empty() {
                    search_names[search_names_index] =
                        Some(SearchName::NotContained(command.to_owned()));
                    search_names_index += 1;
                }
            } else {
                let command = command.trim();

                if !command.is_empty() {
                    search_names[search_names_index] =
                        Some(SearchName::Contained(command.to_owned()));
                    search_names_index += 1;
                }
            }
        }

        Self {
            bone_count,
            mesh_count,
            frame_count,
            frame_rate,
            width,
            height,
            channels,
            search_names,
        }
    }

    /// Determines if the given asset matches this search command.
    #[inline(always)]
    pub fn matches(&self, asset: SearchAsset) -> bool {
        if asset.bone_count > self.bone_count.max || asset.bone_count < self.bone_count.min {
            return false;
        }
        if asset.mesh_count > self.mesh_count.max || asset.mesh_count < self.mesh_count.min {
            return false;
        }
        if asset.frame_count > self.frame_count.max || asset.frame_count < self.frame_count.min {
            return false;
        }
        if asset.frame_rate > self.frame_rate.max || asset.frame_rate < self.frame_rate.min {
            return false;
        }
        if asset.width > self.width.max || asset.width < self.width.min {
            return false;
        }
        if asset.height > self.height.max || asset.height < self.height.min {
            return false;
        }
        if asset.channels > self.channels.max || asset.channels < self.channels.min {
            return false;
        }

        let asset_name = asset.name.to_lowercase();

        let mut names = self.search_names.iter();

        while let Some(Some(name)) = names.next() {
            match name {
                SearchName::Contained(name) => {
                    if !asset_name.contains(name.as_str()) {
                        return false;
                    }
                }
                SearchName::NotContained(name) => {
                    if asset_name.contains(name.as_str()) {
                        return false;
                    }
                }
            }
        }

        true
    }
}

/// Parses a search number into a search range.
#[inline(always)]
fn parse_search_number(number: &str, range: &mut SearchRange) -> Result<(), ParseIntError> {
    if number.is_empty() {
        return Ok(());
    }

    if let Some(number) = number.strip_prefix(">=") {
        range.min = number.parse()?;
    } else if let Some(number) = number.strip_prefix("<=") {
        range.max = number.parse()?;
    } else if let Some(number) = number.strip_prefix('>') {
        let number: u32 = number.parse()?;

        range.min = number.saturating_add(1);
    } else if let Some(number) = number.strip_prefix('<') {
        let number: u32 = number.parse()?;

        range.max = number.saturating_sub(1);
    } else {
        let number: u32 = number.parse()?;

        range.min = number;
        range.max = number;
    }

    Ok(())
}
