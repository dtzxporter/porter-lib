/// Audio formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// Placeholder for an unknown audio format.
    Unknown,
    /// Pulse code modulation: https://wiki.multimedia.cx/index.php/PCM
    IntegerPcm,
    /// MS ADPCM: https://wiki.multimedia.cx/index.php/Microsoft_ADPCM
    MsAdpcm,
    /// Pulse code modulation: https://wiki.multimedia.cx/index.php/PCM
    FloatPcm,
    /// Wwise, custom vorbis: https://wiki.multimedia.cx/index.php/Vorbis
    WwiseVorbis,
}

impl AudioFormat {
    /// Whether or not the audio format is compressed.
    pub const fn is_compressed(&self) -> bool {
        matches!(self, Self::MsAdpcm | Self::WwiseVorbis)
    }

    /// Whether or not the audio format is a coercible version of the given format.
    pub const fn is_coercible(&self, format: Self) -> bool {
        matches!(
            (self, format),
            (Self::IntegerPcm, Self::FloatPcm) | (Self::FloatPcm, Self::IntegerPcm)
        )
    }

    /// Whether or not the audio format is compressible.
    pub const fn is_compressible(&self) -> bool {
        false
    }
}
