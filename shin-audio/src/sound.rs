use crate::resampler::Resampler;
use crate::AudioData;
use kira::clock::clock_info::ClockInfoProvider;
use kira::dsp::Frame;
use kira::sound::Sound;
use kira::track::TrackId;
use ringbuf::HeapConsumer;
use shin_core::format::audio::{AudioDecoder, AudioFile, AudioInfo, AudioSource};
use shin_core::time::{Ticks, Tween, Tweener};
use shin_core::vm::command::types::{AudioWaitStatus, Pan, Volume};
use std::f32::consts::SQRT_2;
use std::sync::atomic::{AtomicI32, AtomicU32};
use std::sync::Arc;
use tracing::debug;

pub const COMMAND_BUFFER_CAPACITY: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    SetVolume(Volume, Tween),
    SetPanning(Pan, Tween),
    Stop(Tween),
}

pub(crate) struct Shared {
    pub wait_status: AtomicI32,
    // TODO: in what unit
    #[allow(unused)] // TODO: use it to implement BGMSYNC (I don't know which unit it uses)
    pub position: AtomicU32,
    // used for lip sync
    pub amplitude: AtomicU32,
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

pub struct SampleProvider {
    source: AudioSource<AudioDecoder<Arc<AudioFile>>>,
    repeat: bool,
    resampler: Resampler,
    fractional_position: f64,
    reached_eof: bool,
}

impl SampleProvider {
    fn new(audio: Arc<AudioFile>, repeat: bool) -> Self {
        Self {
            source: AudioSource::new(
                AudioDecoder::new(audio).expect("Could not create audio decoder"),
            ),
            repeat,
            resampler: Resampler::new(0),
            fractional_position: 0.0,
            reached_eof: false,
        }
    }

    fn audio_info(&self) -> &AudioInfo {
        self.source.inner().audio_info()
    }

    fn push_frame_to_resampler(&mut self) {
        let frame = match self.source.read_sample() {
            Some((left, right)) => Frame { left, right },
            None => {
                if self.repeat {
                    self.source
                        .samples_seek(self.audio_info().loop_start)
                        .expect("Could not seek to loop start");

                    return self.push_frame_to_resampler();
                } else {
                    self.reached_eof = true;
                    Frame::ZERO
                }
            }
        };

        let next_sample_index = self.source.current_samples_position();
        self.resampler.push_frame(frame, next_sample_index - 1);
    }

    fn next(&mut self, dt: f64) -> Frame {
        let out = self.resampler.get(self.fractional_position as f32);
        self.fractional_position += dt * self.source.sample_rate() as f64;
        while self.fractional_position >= 1.0 {
            self.fractional_position -= 1.0;
            self.push_frame_to_resampler();
        }

        out
    }
}

pub struct AudioSound {
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
    pub fn new(data: AudioData, command_consumer: HeapConsumer<Command>) -> Self {
        debug!("Creating audio sound for track {:?}", data.settings.track);

        let mut volume_fade = Tweener::new(0.0);
        volume_fade.enqueue_now(1.0, data.settings.fade_in);

        let shared = Arc::new(Shared::new());

        AudioSound {
            track_id: data.settings.track,
            command_consumer,
            shared: shared.clone(),
            state: PlaybackState::Playing,
            volume: Tweener::new(data.settings.volume.0),
            panning: Tweener::new(data.settings.pan.0),
            volume_fade,
            sample_provider: SampleProvider::new(data.file, data.settings.repeat),
        }
    }

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

    pub(crate) fn shared(&self) -> Arc<Shared> {
        self.shared.clone()
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
