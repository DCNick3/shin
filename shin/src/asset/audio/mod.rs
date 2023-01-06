mod resampler;

use crate::asset::audio::resampler::Resampler;
use crate::asset::Asset;
use anyhow::Context;
use kira::clock::clock_info::ClockInfoProvider;
use kira::dsp::Frame;
use kira::sound::{Sound, SoundData};
use kira::track::TrackId;
use kira::tween::{Tween, Tweener};
use kira::Volume;
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};
use shin_core::format::audio::{read_audio, AudioDecoder, AudioFile};
use std::sync::Arc;

pub struct Audio(AudioFile);

pub struct AudioParams {
    pub track: TrackId,
}

impl Audio {
    pub fn to_kira_data(self: Arc<Self>, params: AudioParams) -> AudioData {
        AudioData(ArcAudio(self), params)
    }
}

impl Asset for Audio {
    fn load_from_bytes(data: Vec<u8>) -> anyhow::Result<Self> {
        read_audio(&data).map(Self).context("Parsing audio file")
    }
}

// more newtypes to the newtype god
struct ArcAudio(Arc<Audio>);

impl AsRef<AudioFile> for ArcAudio {
    fn as_ref(&self) -> &AudioFile {
        &self.0 .0
    }
}

const COMMAND_BUFFER_CAPACITY: usize = 8;

/// Unfortunately, it's not possible to implement SoundData for Arc<AudioData>, so we use a newtype
pub struct AudioData(ArcAudio, AudioParams);

#[derive(Debug, Clone, Copy, PartialEq)]
enum Command {
    SetVolume(Volume, Tween),
    SetPanning(f64, Tween),
    Stop(Tween),
    // TODO: how should BGMWAIT be implemented
}

pub struct AudioHandle {
    command_producer: HeapProducer<Command>,
}

impl SoundData for AudioData {
    type Error = anyhow::Error;
    type Handle = AudioHandle;

    fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
        let (sound, handle) = self.split();
        Ok((Box::new(sound), handle))
    }
}

impl AudioData {
    fn split(self) -> (AudioSound, AudioHandle) {
        let (command_producer, command_consumer) = HeapRb::new(COMMAND_BUFFER_CAPACITY).split();
        let sound = AudioSound {
            track_id: self.1.track,
            command_consumer,
            state: PlaybackState::Playing,
            volume: Tweener::new(Volume::Amplitude(1.0)),
            panning: Tweener::new(0.0),
            volume_fade: Tweener::new(Volume::Amplitude(1.0)),
            sample_provider: SampleProvider::new(self.0),
        };
        (sound, AudioHandle { command_producer })
    }
}

/// The playback state of a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaybackState {
    /// The sound is playing normally.
    Playing,
    /// The sound is fading out, and when the fade-out
    /// is finished, playback will stop.
    Stopping,
    /// The sound has stopped and can no longer be resumed.
    Stopped,
}

struct SampleProvider {
    decoder: AudioDecoder<ArcAudio>,
    resampler: Resampler,
    buffer_offset: usize,
    fractional_position: f64,
    end_of_file: bool,
}

impl SampleProvider {
    fn new(audio: ArcAudio) -> Self {
        Self {
            decoder: AudioDecoder::new(audio).expect("Could not create audio decoder"),
            resampler: Resampler::new(0),
            buffer_offset: 0,
            fractional_position: 0.0,
            end_of_file: false,
        }
    }

    fn position(&self) -> i64 {
        // TODO: seeking???
        self.decoder.samples_position() + self.buffer_offset as i64
    }

    fn push_next_frame(&mut self) {
        let buffer = self.decoder.buffer();
        let buffer = &buffer[self.buffer_offset * 2..];
        if !buffer.is_empty() {
            // TODO: handle non-stereo audio?
            self.buffer_offset += 1;
            self.resampler.push_frame(
                Frame {
                    left: buffer[0],
                    right: buffer[1],
                },
                self.position(),
            );
        } else {
            match self.decoder.decode_frame() {
                Some(pos) => self.buffer_offset = pos,
                None => {
                    // TODO: start outputting silence instead of just stopping?
                    self.end_of_file = true;
                }
            }

            self.push_next_frame()
        }
    }

    fn next(&mut self, dt: f64) -> Option<Frame> {
        let out = self.resampler.get(self.fractional_position as f32);
        self.fractional_position += dt * self.decoder.info().sample_rate as f64;
        while self.fractional_position >= 1.0 {
            self.fractional_position -= 1.0;
            self.push_next_frame();
        }

        if self.end_of_file {
            None
        } else {
            Some(out)
        }
    }
}

struct AudioSound {
    track_id: TrackId,
    command_consumer: HeapConsumer<Command>,
    state: PlaybackState,
    volume: Tweener<Volume>,
    panning: Tweener,
    volume_fade: Tweener<Volume>,
    sample_provider: SampleProvider,
}
impl Sound for AudioSound {
    fn track(&mut self) -> TrackId {
        self.track_id
    }

    fn process(&mut self, dt: f64, _clock_info_provider: &ClockInfoProvider) -> Frame {
        match self.sample_provider.next(dt) {
            None => todo!(),
            Some(f) => f,
        }
    }

    fn finished(&self) -> bool {
        self.state == PlaybackState::Stopped
    }
}
