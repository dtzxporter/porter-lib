#![allow(dead_code)]

use std::time::Duration;

use porter_audio::Audio;
use porter_audio::AudioFormat;

/// A cross platform audio player.
pub struct AudioPlayer {
    state: State,
}

/// Errors that occur in the audio player.
pub enum AudioPlayerError {
    Unsupported,
    Error,
}

impl AudioPlayer {
    /// Loads the given audio asset into a new audio player.
    ///
    /// The asset must be in the [`AudioFormat::IntegerPcm`] format.
    pub fn load(audio: Audio) -> Result<Self, AudioPlayerError> {
        if !matches!(audio.format(), AudioFormat::IntegerPcm) {
            return Err(AudioPlayerError::Error);
        }

        Ok(Self {
            state: load_audio(audio)?,
        })
    }

    /// Returns the number of channels for this audio player.
    pub fn channels(&self) -> u32 {
        self.state.channels()
    }

    /// Returns the sample rate for this audio player.
    pub fn sample_rate(&self) -> u32 {
        self.state.sample_rate()
    }

    /// Starts, resumes, or restarts the audio player.
    pub fn play(&mut self) {
        self.state.play();
    }

    /// Whether or not the audio player is playing audio.
    pub fn is_playing(&self) -> bool {
        self.state.is_playing()
    }

    /// Pauses the audio player.
    pub fn pause(&mut self) {
        self.state.pause();
    }

    /// Sets the volume of the player.
    pub fn volume(&mut self, volume: u32) {
        self.state.volume(volume);
    }

    /// Seeks the audio player.
    pub fn seek(&mut self, position: Duration) {
        self.state.seek(position);
    }

    /// Returns the position of the audio player.
    pub fn position(&self) -> Duration {
        self.state.position()
    }

    /// Returns the duration of the audio player.
    pub fn duration(&self) -> Duration {
        self.state.duration()
    }
}

#[cfg(all(target_os = "windows", feature = "sounds-convertible"))]
use windows::*;

#[cfg(any(not(target_os = "windows"), not(feature = "sounds-convertible")))]
use unsupported::*;

#[cfg(all(target_os = "windows", feature = "sounds-convertible"))]
mod windows {
    use std::time::Duration;

    use porter_audio::Audio;

    use windows_sys::Win32::Media::Audio::*;
    use windows_sys::Win32::Media::KernelStreaming::*;
    use windows_sys::Win32::Media::*;

    use super::AudioPlayerError;

    /// Internal state of the windows audio player.
    pub struct State {
        audio: Audio,
        header: Box<WAVEHDR>,
        handle: HWAVEOUT,
        offset: usize,
        paused: bool,
    }

    /// Loads an audio sample on the windows platform.
    pub fn load_audio(audio: Audio) -> Result<State, AudioPlayerError> {
        let channels = audio.channels();

        let bits_per_sample = audio.bits_per_sample();
        let block_align = audio
            .block_align()
            .unwrap_or_else(|| (channels * bits_per_sample) / 8);

        let sample_rate = audio.sample_rate();
        let avg_bytes_per_sec = sample_rate * block_align;

        let format = WAVEFORMATEXTENSIBLE {
            Format: WAVEFORMATEX {
                wFormatTag: WAVE_FORMAT_EXTENSIBLE as _,
                nChannels: channels as _,
                nSamplesPerSec: sample_rate,
                nAvgBytesPerSec: avg_bytes_per_sec,
                nBlockAlign: block_align as _,
                wBitsPerSample: bits_per_sample as _,
                cbSize: (size_of::<WAVEFORMATEXTENSIBLE>() - size_of::<WAVEFORMATEX>()) as _,
            },
            Samples: WAVEFORMATEXTENSIBLE_0 {
                wValidBitsPerSample: bits_per_sample as _,
            },
            dwChannelMask: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT,
            SubFormat: KSDATAFORMAT_SUBTYPE_PCM,
        };

        let mut handle: HWAVEOUT = std::ptr::null_mut();

        // SAFETY:
        // The format is initialized fully, we're requesting automatic device selection with no callback.
        // All error states are handled properly below, and we keep the handle alive in self.
        let result = unsafe {
            waveOutOpen(
                &mut handle,
                WAVE_MAPPER,
                &format as *const _ as _,
                0,
                0,
                CALLBACK_NULL,
            )
        };

        if result == MMSYSERR_NOMEM || result == MMSYSERR_NODRIVER || result == WAVERR_BADFORMAT {
            return Err(AudioPlayerError::Unsupported);
        } else if result != MMSYSERR_NOERROR {
            return Err(AudioPlayerError::Error);
        }

        let header: Box<WAVEHDR> = Box::new(WAVEHDR {
            lpData: audio.data().as_ptr() as _,
            dwBufferLength: audio.data().len() as _,
            ..Default::default()
        });

        let mut state = State {
            audio,
            header,
            handle,
            offset: 0,
            paused: true,
        };

        state.pause();
        state.prepare_header()?;

        Ok(state)
    }

    impl State {
        /// Returns the channel count of this audio handle.
        pub fn channels(&self) -> u32 {
            self.audio.channels()
        }

        /// Returns the sample rate of this audio handle.
        pub fn sample_rate(&self) -> u32 {
            self.audio.sample_rate()
        }

        // Plays this audio handle.
        pub fn play(&mut self) {
            if self.audio.data().len() == self.byte_position() {
                self.seek(Duration::ZERO);
            }

            // SAFETY:
            // Handle is valid for the lifetime of the state, and it's safe to call restart multiple times.
            unsafe { waveOutRestart(self.handle) };

            self.paused = false;
        }

        /// Whether or not the audio handle is playing.
        pub fn is_playing(&self) -> bool {
            !self.paused && self.audio.data().len() != self.byte_position()
        }

        // Pauses this audio handle.
        pub fn pause(&mut self) {
            // SAFETY:
            // Handle is valid for the lifetime of the state, and it's safe to call pause multiple times.
            unsafe { waveOutPause(self.handle) };

            self.paused = true;
        }

        /// Sets the volume of this audio handle.
        pub fn volume(&mut self, volume: u32) {
            let volume: u32 = (volume * 0x7FFF) / 50;
            let volume = volume | (volume << 16);

            unsafe { waveOutSetVolume(self.handle, volume) };
        }

        /// Seeks to the given position in this audio handle.
        pub fn seek(&mut self, position: Duration) {
            self.unprepare_header();

            let offset = self.audio.offset(position).unwrap_or_default();

            let data = self.audio.data().as_ptr();
            let length = self.audio.data().len() - offset;

            *self.header = WAVEHDR {
                // SAFETY:
                // The audio offset method ensures that it's never outside of the bounds of the data array.
                lpData: unsafe { data.byte_add(offset) } as _,
                dwBufferLength: length as _,
                ..Default::default()
            };

            // Maintain paused state, when we unprepare the previous header, it takes paused state with it.
            if self.paused {
                self.pause();
            }

            if self.prepare_header().is_ok() {
                self.offset = offset;
            }
        }

        /// Returns the position of this handle.
        pub fn position(&self) -> Duration {
            self.audio
                .position(self.byte_position())
                .unwrap_or_default()
        }

        /// Returns the duration of this handle.
        pub fn duration(&self) -> Duration {
            self.audio.duration().unwrap_or_default()
        }

        /// Gets the byte offset for the current position of this handle.
        fn byte_position(&self) -> usize {
            let mut time = MMTIME {
                wType: TIME_BYTES,
                ..Default::default()
            };

            // SAFETY:
            // We properly initialized the time structure and set a type before calling.
            // This api can accept an invalid handle, and since we initialize to 0, we can ignore errors.
            unsafe { waveOutGetPosition(self.handle, &mut time, size_of::<MMTIME>() as _) };

            // SAFETY:
            // We initialized the time structure with zeros and set the proper bytes type.
            unsafe { time.u.cb as usize + self.offset }
        }

        /// Prepares the current header and sends it off to the audio driver.
        fn prepare_header(&mut self) -> Result<(), AudioPlayerError> {
            // SAFETY:
            // The header is initialized properly before this call.
            // This gracefully handles headers that were already prepared as a no-op.
            let result = unsafe {
                waveOutPrepareHeader(self.handle, self.header.as_mut(), size_of::<WAVEHDR>() as _)
            };

            if result != MMSYSERR_NOERROR {
                return Err(AudioPlayerError::Error);
            }

            // SAFETY:
            // We ensure that the header has previously been prepared successfully in the call above.
            // The header lives for as long as the handle does because it's stored in self.
            let result = unsafe {
                waveOutWrite(self.handle, self.header.as_mut(), size_of::<WAVEHDR>() as _)
            };

            if result != MMSYSERR_NOERROR {
                return Err(AudioPlayerError::Error);
            }

            Ok(())
        }

        /// Unprepares and releases all resources that are using the current header.
        fn unprepare_header(&mut self) {
            // SAFETY:
            // In order to clean up, we must make sure that all buffers are released by the driver.
            // Reset stops playback and marks all buffers as released.
            unsafe { waveOutReset(self.handle) };

            // SAFETY:
            // We must unprepare the previously prepared header, removing it from the driver.
            // This method is a no-op when the header hasn't been prepared.
            unsafe {
                waveOutUnprepareHeader(self.handle, self.header.as_mut(), size_of::<WAVEHDR>() as _)
            };
        }
    }

    impl Drop for State {
        fn drop(&mut self) {
            // Unprepare header to ensure that resources are released before closing the handle.
            self.unprepare_header();

            // SAFETY:
            // After the header has been unprepared we can safely free the wave out handle.
            // The handle must be valid at this point, otherwise we have no state.
            unsafe { waveOutClose(self.handle) };
        }
    }
}

#[cfg(any(not(target_os = "windows"), not(feature = "sounds-convertible")))]
mod unsupported {
    use std::time::Duration;

    use porter_audio::Audio;

    use super::AudioPlayerError;

    /// Internal state of the unsupported platform.
    pub struct State;

    /// Returns unsupported for unsupported audio preview platforms.
    pub fn load_audio(audio: Audio) -> Result<State, AudioPlayerError> {
        let _ = audio;

        Err(AudioPlayerError::Unsupported)
    }

    impl State {
        /// Returns 0 channels for unsupported platforms.
        pub fn channels(&self) -> u32 {
            0
        }

        /// Returns 0 rate for unsupported platforms.
        pub fn sample_rate(&self) -> u32 {
            0
        }

        /// Noop for unsupported platforms.
        pub fn play(&mut self) {}

        /// Returns false for unsupported platforms.
        pub fn is_playing(&self) -> bool {
            false
        }

        /// Noop for unsupported platforms.
        pub fn pause(&mut self) {}

        /// Noop for unsupported platforms.
        pub fn volume(&mut self, volume: u32) {
            let _ = volume;
        }

        /// Noop for unsupported platforms.
        pub fn seek(&mut self, position: Duration) {
            let _ = position;
        }

        /// Returns zero duration for unsupported platforms.
        pub fn position(&self) -> Duration {
            Duration::ZERO
        }

        /// Returns zero duration for unsupported platforms.
        pub fn duration(&self) -> Duration {
            Duration::ZERO
        }
    }
}
