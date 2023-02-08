//! Implements the SoundData trait for the Kira audio library.

use anyhow::{anyhow, Result};
use kira::clock::clock_info::ClockInfoProvider;
use kira::dsp::Frame;
use kira::sound::{Sound, SoundData};
use kira::track::TrackId;
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};
use shin_core::format::audio::{AudioDecoder, AudioDecoderIterator, AudioFile};
use shin_core::time::{Ticks, Tween, Tweener};
use shin_core::vm::command::types::{AudioWaitStatus, Pan, Volume};
use std::f32::consts::SQRT_2;
use std::sync::atomic::{AtomicI32, AtomicU32};
use std::sync::Arc;
use tracing::debug;

use super::resampler::Resampler;
use super::{Audio, AudioParams};

impl Audio {
    pub fn to_kira_data(self: Arc<Self>, params: AudioParams) -> AudioData {
        AudioData(ArcAudio(self), params)
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
    SetPanning(Pan, Tween),
    Stop(Tween),
}

struct Shared {
    wait_status: AtomicI32,
    // TODO: in what unit
    #[allow(unused)] // TODO: use it to implement BGMSYNC (I don't know which unit it uses)
    position: AtomicU32,
    // used for lip sync
    amplitude: AtomicU32,
}

impl Shared {
    fn new() -> Self {
        Self {
            wait_status: AtomicI32::new(0),
            position: AtomicU32::new(0),
            amplitude: AtomicU32::new(0),
        }
    }
}

pub struct AudioHandle {
    command_producer: HeapProducer<Command>,
    shared: Arc<Shared>,
}

impl AudioHandle {
    pub fn get_wait_status(&self) -> AudioWaitStatus {
        AudioWaitStatus::from_bits_truncate(
            self.shared
                .wait_status
                .load(std::sync::atomic::Ordering::SeqCst),
        )
    }

    #[allow(unused)] // TODO: use it for lip-sync
    pub fn get_amplitude(&self) -> f32 {
        f32::from_bits(
            self.shared
                .amplitude
                .load(std::sync::atomic::Ordering::SeqCst),
        )
    }

    /// Sets the volume of the sound.
    /// The volume is a value between 0.0 and 1.0, on the linear scale.
    pub fn set_volume(&mut self, volume: Volume, tween: Tween) -> Result<()> {
        self.command_producer
            .push(Command::SetVolume(volume, tween))
            .map_err(|_| anyhow!("Command queue full"))
    }

    /// Sets the panning of the sound
    pub fn set_panning(&mut self, panning: Pan, tween: Tween) -> Result<()> {
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

        debug!("Creating audio sound for track {:?}", self.1.track);

        let mut volume_fade = Tweener::new(0.0);
        volume_fade.enqueue_now(1.0, self.1.fade_in);

        let shared = Arc::new(Shared::new());
        let sound = AudioSound {
            track_id: self.1.track,
            command_consumer,
            shared: shared.clone(),
            state: PlaybackState::Playing,
            volume: Tweener::new(self.1.volume.0),
            panning: Tweener::new(self.1.pan.0),
            volume_fade,
            sample_provider: SampleProvider::new(self.0, self.1.repeat),
        };
        (
            sound,
            AudioHandle {
                command_producer,
                shared,
            },
        )
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
    decoder: AudioDecoderIterator<ArcAudio>,
    repeat: bool,
    resampler: Resampler,
    fractional_position: f64,
    reached_eof: bool,
}

impl SampleProvider {
    fn new(audio: ArcAudio, repeat: bool) -> Self {
        Self {
            decoder: AudioDecoderIterator::new(
                AudioDecoder::new(audio).expect("Could not create audio decoder"),
            ),
            repeat,
            resampler: Resampler::new(0),
            fractional_position: 0.0,
            reached_eof: false,
        }
    }

    fn push_frame_to_resampler(&mut self) {
        let frame = match self.decoder.next() {
            Some((left, right)) => Frame { left, right },
            None => {
                if self.repeat {
                    self.decoder.seek(self.decoder.info().loop_start as u64);

                    return self.push_frame_to_resampler();
                } else {
                    self.reached_eof = true;
                    Frame::ZERO
                }
            }
        };

        self.resampler.push_frame(frame, self.decoder.position());
    }

    fn next(&mut self, dt: f64) -> Frame {
        let out = self.resampler.get(self.fractional_position as f32);
        self.fractional_position += dt * self.decoder.info().sample_rate as f64;
        while self.fractional_position >= 1.0 {
            self.fractional_position -= 1.0;
            self.push_frame_to_resampler();
        }

        out
    }
}

struct AudioSound {
    track_id: TrackId,
    command_consumer: HeapConsumer<Command>,
    shared: Arc<Shared>,
    state: PlaybackState,
    volume: Tweener,
    panning: Tweener,
    volume_fade: Tweener,
    sample_provider: SampleProvider,
}

impl AudioSound {
    fn stop(&mut self, fade_out_tween: Tween) {
        self.state = PlaybackState::Stopping;
        self.volume_fade.enqueue_now(0.0, fade_out_tween);
    }

    fn wait_status(&self) -> AudioWaitStatus {
        let mut result = AudioWaitStatus::empty();

        if self.state == PlaybackState::Stopped {
            result |= AudioWaitStatus::STOPPED;
        }
        if self.state == PlaybackState::Playing {
            result |= AudioWaitStatus::PLAYING;
        }
        if self.volume.is_idle() {
            result |= AudioWaitStatus::VOLUME_TWEENER_IDLE;
        }
        if self.panning.is_idle() {
            result |= AudioWaitStatus::PANNING_TWEENER_IDLE;
        }
        result |= AudioWaitStatus::PLAY_SPEED_TWEENER_IDLE;

        result
    }
}

impl Sound for AudioSound {
    fn track(&mut self) -> TrackId {
        self.track_id
    }

    fn on_start_processing(&mut self) {
        while let Some(command) = self.command_consumer.pop() {
            match command {
                // note: unlike in the layer props, we do the "enqueue_now" thing here
                // bacause we don't want to wait for previous audio changes to be applied
                // ideally, this should never allocate the tweener queue
                Command::SetVolume(volume, tween) => self.volume.enqueue_now(volume.0, tween),
                Command::SetPanning(panning, tween) => self.panning.enqueue_now(panning.0, tween),
                Command::Stop(tween) => self.stop(tween),
            }
        }

        self.shared.wait_status.store(
            self.wait_status().bits(),
            std::sync::atomic::Ordering::SeqCst,
        );
        // TODO: compute the amplitude
        // TODO: provide the position
    }

    fn process(&mut self, dt: f64, _clock_info_provider: &ClockInfoProvider) -> Frame {
        let dt_ticks = Ticks::from_seconds(dt as f32);

        // update tweeners
        self.volume.update(dt_ticks);
        self.panning.update(dt_ticks);
        self.volume_fade.update(dt_ticks);

        if self.state == PlaybackState::Stopping && self.volume_fade.is_idle() {
            self.state = PlaybackState::Stopped
        }

        let mut f = self.sample_provider.next(dt);

        if self.sample_provider.reached_eof {
            self.state = PlaybackState::Stopped;
        }

        let pan = self.panning.value();
        let volume = self.volume_fade.value() * self.volume.value();

        f *= volume;
        if pan != 0.0 {
            f = Frame::new(f.left * (1.0 - pan).sqrt(), f.right * pan.sqrt()) * SQRT_2
        }

        f
    }

    fn finished(&self) -> bool {
        self.state == PlaybackState::Stopped && self.sample_provider.resampler.outputting_silence()
    }
}
