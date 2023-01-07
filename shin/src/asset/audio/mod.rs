mod resampler;

use crate::asset::audio::resampler::Resampler;
use crate::asset::Asset;
use anyhow::Context;
use anyhow::{anyhow, Result};
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
    pub volume: f32,
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

impl AudioHandle {
    /// Sets the volume of the sound (as a factor of the original volume).
    pub fn set_volume(&mut self, volume: impl Into<Volume>, tween: Tween) -> Result<()> {
        self.command_producer
            .push(Command::SetVolume(volume.into(), tween))
            .map_err(|_| anyhow!("Command queue full"))
    }

    /// Sets the panning of the sound, where `0.0` is hard left,
    /// `0.5` is center, and `1.0` is hard right.
    pub fn set_panning(&mut self, panning: f64, tween: Tween) -> Result<()> {
        self.command_producer
            .push(Command::SetPanning(panning, tween))
            .map_err(|_| anyhow!("Command queue full"))
    }

    /// Fades out the sound to silence with the given tween and then
    /// stops playback.
    ///
    /// Once the sound is stopped, it cannot be restarted.
    pub fn stop(&mut self, tween: Tween) -> Result<()> {
        self.command_producer
            .push(Command::Stop(tween))
            .map_err(|_| anyhow!("Command queue full"))
    }
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
            volume: Tweener::new(Volume::Amplitude(self.1.volume as f64)),
            panning: Tweener::new(0.5),
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

impl AudioSound {
    fn stop(&mut self, fade_out_tween: Tween) {
        self.state = PlaybackState::Stopping;
        self.volume_fade
            .set(Volume::Decibels(Volume::MIN_DECIBELS), fade_out_tween);
    }
}

impl Sound for AudioSound {
    fn track(&mut self) -> TrackId {
        self.track_id
    }

    fn on_start_processing(&mut self) {
        while let Some(command) = self.command_consumer.pop() {
            match command {
                Command::SetVolume(volume, tween) => self.volume.set(volume, tween),
                Command::SetPanning(panning, tween) => self.panning.set(panning, tween),
                Command::Stop(tween) => self.stop(tween),
            }
        }
    }

    fn process(&mut self, dt: f64, clock_info_provider: &ClockInfoProvider) -> Frame {
        // update tweeners
        self.volume.update(dt, clock_info_provider);
        self.panning.update(dt, clock_info_provider);
        if self.volume_fade.update(dt, clock_info_provider) && self.state == PlaybackState::Stopping
        {
            self.state = PlaybackState::Stopped
        }

        match self.sample_provider.next(dt) {
            None => todo!("finish playing or loop around"),
            Some(f) => (f
                * self.volume_fade.value().as_amplitude() as f32
                * self.volume.value().as_amplitude() as f32)
                .panned(self.panning.value() as f32),
        }
    }

    fn finished(&self) -> bool {
        self.state == PlaybackState::Stopped
    }
}
