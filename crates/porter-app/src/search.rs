use std::num::ParseIntError;
use std::time::Duration;

/// Ways to filter on a number range.
#[derive(Debug, Clone, Copy)]
struct SearchRange {
    min: u16,
    max: u16,
}

impl Default for SearchRange {
    fn default() -> Self {
        Self {
            min: u16::MIN,
            max: u16::MAX,
        }
    }
}

/// Ways to filter on a duration.
#[derive(Debug, Clone, Copy)]
struct SearchDuration {
    min: Duration,
    max: Duration,
}

impl Default for SearchDuration {
    fn default() -> Self {
        Self {
            min: Duration::ZERO,
            max: Duration::MAX,
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
    bone_count: u16,
    mesh_count: u16,
    frame_count: u16,
    frame_rate: u16,
    width: u16,
    height: u16,
    channels: u16,
    duration: Duration,
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
            duration: Duration::ZERO,
            name,
        }
    }

    /// Sets the count of bones this asset has.
    #[inline]
    pub fn bone_count<C: TryInto<u16>>(mut self, count: C) -> Self {
        self.bone_count = count.try_into().unwrap_or_default();
        self
    }

    /// Sets the count of meshes this asset has.
    #[inline]
    pub fn mesh_count<C: TryInto<u16>>(mut self, count: C) -> Self {
        self.mesh_count = count.try_into().unwrap_or_default();
        self
    }

    /// Sets the count of frames this asset has.
    #[inline]
    pub fn frame_count<C: TryInto<u16>>(mut self, count: C) -> Self {
        self.frame_count = count.try_into().unwrap_or_default();
        self
    }

    /// Sets the frame rate this asset has.
    #[inline]
    pub fn frame_rate<C: TryInto<u16>>(mut self, rate: C) -> Self {
        self.frame_rate = rate.try_into().unwrap_or_default();
        self
    }

    /// Sets the width this asset has.
    #[inline]
    pub fn width<C: TryInto<u16>>(mut self, width: C) -> Self {
        self.width = width.try_into().unwrap_or_default();
        self
    }

    /// Sets the height this asset has.
    #[inline]
    pub fn height<C: TryInto<u16>>(mut self, height: C) -> Self {
        self.height = height.try_into().unwrap_or_default();
        self
    }

    /// Sets the channels this asset has.
    #[inline]
    pub fn channels<C: TryInto<u16>>(mut self, channels: C) -> Self {
        self.channels = channels.try_into().unwrap_or_default();
        self
    }

    /// Sets the duration this asset has.
    pub const fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
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
    duration: SearchDuration,
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
        let mut duration = SearchDuration::default();

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
            } else if let Some(command) = command.strip_prefix("duration:") {
                let _ = parse_search_duration(command, &mut duration);
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
            duration,
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
        if asset.duration > self.duration.max || asset.duration < self.duration.min {
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
        let number: u16 = number.parse()?;

        range.min = number.saturating_add(1);
    } else if let Some(number) = number.strip_prefix('<') {
        let number: u16 = number.parse()?;

        range.max = number.saturating_sub(1);
    } else {
        let number: u16 = number.parse()?;

        range.min = number;
        range.max = number;
    }

    Ok(())
}

/// Parses a search duration into its range.
#[inline(always)]
fn parse_search_duration(
    mut duration: &str,
    range: &mut SearchDuration,
) -> Result<(), ParseIntError> {
    if duration.is_empty() {
        return Ok(());
    }

    let scale_factor = if let Some(result) = duration.strip_suffix("ms") {
        duration = result;
        1
    } else if let Some(result) = duration.strip_suffix("s") {
        duration = result;
        1000
    } else if let Some(result) = duration.strip_suffix("m") {
        duration = result;
        1000 * 60
    } else if let Some(result) = duration.strip_suffix("h") {
        duration = result;
        1000 * 3600
    } else {
        1
    };

    if let Some(duration) = duration.strip_prefix(">=") {
        range.min = Duration::from_millis(duration.parse::<u64>()? * scale_factor);
    } else if let Some(duration) = duration.strip_prefix("<=") {
        range.max = Duration::from_millis(duration.parse::<u64>()? * scale_factor);
    } else if let Some(duration) = duration.strip_prefix(">") {
        let duration = Duration::from_millis(duration.parse::<u64>()? * scale_factor);

        range.min = duration.saturating_add(Duration::from_millis(1));
    } else if let Some(duration) = duration.strip_prefix("<") {
        let duration = Duration::from_millis(duration.parse::<u64>()? * scale_factor);

        range.max = duration.saturating_sub(Duration::from_millis(1));
    } else {
        let duration = Duration::from_millis(duration.parse::<u64>()? * scale_factor);

        range.min = duration;
        range.max = duration;
    }

    Ok(())
}
