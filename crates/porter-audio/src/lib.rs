#![allow(unstable_name_collisions)]

mod audio;
mod audio_file_type;
mod audio_format;
mod error;
mod software_coerce;
mod software_compress;
mod software_decompress;

pub(crate) mod audio_file_type_flac;
pub(crate) mod audio_file_type_wav;

#[cfg(feature = "ogg")]
pub(crate) mod audio_file_type_ogg;

pub use audio::*;
pub use audio_file_type::*;
pub use audio_format::*;
pub use error::*;

pub(crate) use software_coerce::*;
pub(crate) use software_compress::*;
pub(crate) use software_decompress::*;

#[cfg(feature = "ms-adpcm")]
mod software_decompress_ms_adpcm;
#[cfg(feature = "raw-flac")]
mod software_decompress_raw_flac;
#[cfg(feature = "wwise-vorbis")]
mod software_decompress_wwise_vorbis;

#[cfg(feature = "ms-adpcm")]
pub(crate) use software_decompress_ms_adpcm::*;
#[cfg(feature = "raw-flac")]
pub(crate) use software_decompress_raw_flac::*;
#[cfg(feature = "wwise-vorbis")]
pub(crate) use software_decompress_wwise_vorbis::*;
