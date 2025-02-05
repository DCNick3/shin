use std::io::{Read, Seek};

use anyhow::{Context, Result};
use glam::{Mat4, Vec4};
use kira::track::TrackId;
use parking_lot::{Mutex, RwLock, RwLockReadGuard, RwLockUpgradableReadGuard};
use shin_audio::{AudioData, AudioManager, AudioSettings};
use shin_core::{
    primitives::{
        exclusive::Exclusive,
        update::{FrameId, UpdateTracker},
    },
    time::{Ticks, Tween},
    vm::command::types::{Pan, Volume},
};
use shin_render::{
    render_pass::RenderPass,
    shaders::types::{buffer::VertexSource, vertices::MovieVertex},
    DrawPrimitive, RenderProgramWithArguments, RenderRequestBuilder,
};
use tracing::{error, info, trace, warn};

use crate::{
    audio::AacFrameSource,
    h264_decoder::{FrameTiming, H264Decoder, H264DecoderTrait, Nv12Frame},
    mp4::Mp4,
    timer::Timer,
    VideoFrameTexture,
};

struct VideoPlayerInner {
    update_tracker: UpdateTracker,
    // we could use [`Exclusive`] instead of [`Mutex`] here, but `set_volume` will then have to take a lock that is much larger than necessary
    timer: Mutex<Timer>,
    video_decoder: Exclusive<H264Decoder>,
    video_texture: VideoFrameTexture,
    pending_frame: Option<(FrameTiming, Nv12Frame)>,
}

pub struct VideoPlayerHandle {
    inner: RwLock<VideoPlayerInner>,
}

impl VideoPlayerHandle {
    pub fn new<S: Read + Seek + Send + 'static>(
        device: &wgpu::Device,
        audio_manager: &AudioManager,
        mp4: Mp4<S>,
    ) -> Result<VideoPlayerHandle> {
        let time_base = mp4
            .video_track
            .get_mp4_track_info(|track| track.timescale());

        let start = std::time::Instant::now();
        let mut video_decoder =
            H264Decoder::new(mp4.video_track).context("Initializing H264Decoder")?;
        let pending_frame = video_decoder.read_frame().context("Reading first frame")?;
        let duration = start.elapsed();

        info!("H264Decoder::new took {:?}", duration);

        let video_texture = VideoFrameTexture::new(
            device,
            video_decoder
                .frame_size()
                .context("Getting H264 frame size")?,
        );

        // TODO: use the audio track
        // if we are using audio the timer should be tracking the audio playback
        let audio_handle = if let Some(track) = mp4.audio_track {
            let frame_source = AacFrameSource::new(track).context("Initializing AacFrameSource")?;
            Some(audio_manager.play(AudioData {
                source: frame_source,
                settings: AudioSettings {
                    track: TrackId::Main,
                    fade_in: Tween::MS_15,
                    loop_start: None,
                    volume: Volume::default(),
                    pan: Pan::default(),
                },
            }))
        } else {
            None
        };

        let timer = match audio_handle {
            Some(handle) => Timer::new_audio_tied(time_base, handle),
            None => Timer::new_independent(time_base),
        };

        let inner = VideoPlayerInner {
            update_tracker: UpdateTracker::new(),
            timer: Mutex::new(timer),
            video_decoder: Exclusive::new(video_decoder),
            video_texture,
            pending_frame,
        };

        Ok(VideoPlayerHandle {
            inner: RwLock::new(inner),
        })
    }

    pub fn set_volume(&self, volume: Volume) {
        let read_guard = self.inner.read();
        let mut timer = read_guard.timer.lock();

        // timer keeps the audio handle, so we need to call it on the timer
        // maybe we should restructure this stuff a bit...
        timer.set_audio_volume(volume);
    }

    pub fn update(&self, game_frame_id: FrameId, delta_time: Ticks, queue: &wgpu::Queue) {
        let read_guard = self.inner.upgradable_read();
        if !read_guard.update_tracker.needs_update(game_frame_id) {
            return;
        }

        let mut write_guard = RwLockUpgradableReadGuard::upgrade(read_guard);

        let this = &mut *write_guard;

        if this.update_tracker.update(game_frame_id) {
            let current_time = {
                let timer = this.timer.get_mut();
                timer.update(delta_time);
                timer.time()
            };

            let mut skipped_frames = 0;
            // find the latest frame that is ready for display
            // this might be the currently pending frame, or any of the frames after it (shouldn't happen often I think)
            while let Some((timing, ref frame)) = this.pending_frame {
                // if it's not time to display the frame yet - stop the loop
                if timing.start_time > current_time {
                    // very noisy
                    // trace!(
                    //     "Not time to display frame #{}, time: {}",
                    //     timing.frame_number,
                    //     timing.start_time
                    // );
                    break;
                }

                // look at the frame after the pending one
                let next_frame = this
                    .video_decoder
                    .get_mut()
                    .read_frame()
                    .unwrap_or_else(|err| {
                        error!("Error reading frame: {}. Stopping playback", err);
                        None
                    });

                // if the next frame is not ready for display yet...
                if next_frame
                    .as_ref()
                    .map_or(true, |(timing, _)| timing.start_time > current_time)
                {
                    if skipped_frames > 0 {
                        warn!("Skipped {} frames", skipped_frames);
                    }

                    // then update the texture with the pending frame
                    trace!(
                        "Displaying frame #{}, time: {}",
                        timing.frame_number,
                        timing.start_time
                    );
                    this.video_texture.write_data_nv12(queue, frame);
                    // the loop will not enter again, so the pending frame will now be displayed
                } else {
                    skipped_frames += 1;
                    // if the next frame is also ready for display, then we should skip the pending frame
                }

                if next_frame.is_none() {
                    info!("No more frames, stopping playback");
                }

                this.pending_frame = next_frame;
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        let read_guard = self.inner.read();
        read_guard.pending_frame.is_none()
    }

    pub fn get_frame(&self) -> Option<VideoPlayerFrameHandle> {
        let read_guard = self.inner.read();

        // TODO: actually, this will eat the last frame of the video
        // we need to provide another mechanism for determining how long to display the last frame
        if read_guard.pending_frame.is_none() {
            return None;
        }

        Some(VideoPlayerFrameHandle { guard: read_guard })
    }
}

pub struct VideoPlayerFrameHandle<'a> {
    guard: RwLockReadGuard<'a, VideoPlayerInner>,
}

impl<'a> VideoPlayerFrameHandle<'a> {
    pub fn get_frame(&self) -> &VideoFrameTexture {
        &self.guard.video_texture
    }

    pub fn render(&self, pass: &mut RenderPass, builder: RenderRequestBuilder, transform: Mat4) {
        let tex = &self.guard.video_texture;

        let [width, height] = tex.get_size().to_array();

        // TODO: handle movie with alpha
        pass.run(builder.build(
            RenderProgramWithArguments::Movie {
                vertices: VertexSource::VertexData {
                    vertices: &[
                        MovieVertex {
                            coords: Vec4::new(0.0, 0.0, 0.0, 0.0),
                        },
                        MovieVertex {
                            coords: Vec4::new(width, 0.0, 1.0, 0.0),
                        },
                        MovieVertex {
                            coords: Vec4::new(0.0, height, 0.0, 1.0),
                        },
                        MovieVertex {
                            coords: Vec4::new(width, height, 1.0, 1.0),
                        },
                    ],
                },
                texture_luma: tex.get_y_source(),
                texture_chroma: tex.get_uv_source(),
                transform,
                color_bias: Vec4::new(0.0625, 0.5, 0.5, 1.1643835),
                color_transform: [
                    Vec4::new(1.1643835, 0.0, 1.7927411, 0.0),
                    Vec4::new(1.1643835, -0.21322097, -0.5328817, 0.0),
                    Vec4::new(1.1643835, 2.1124017, 0.0, 0.0),
                ],
            },
            DrawPrimitive::TrianglesStrip,
        ))
    }
}
