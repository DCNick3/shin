use std::io::{Read, Seek};

use anyhow::{Context, Result};
use glam::{Mat4, Vec3, Vec4};
use kira::track::TrackId;
use shin_audio::{AudioData, AudioManager, AudioSettings};
use shin_core::{
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

pub struct VideoPlayer {
    timer: Timer,
    video_decoder: H264Decoder,
    video_texture: VideoFrameTexture,
    pending_frame: Option<(FrameTiming, Nv12Frame)>,
}

impl VideoPlayer {
    pub fn new<S: Read + Seek + Send + 'static>(
        device: &wgpu::Device,
        audio_manager: &AudioManager,
        mp4: Mp4<S>,
    ) -> Result<VideoPlayer> {
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

        Ok(VideoPlayer {
            timer,
            video_decoder,
            video_texture,
            pending_frame,
        })
    }

    pub fn update(&mut self, delta_time: Ticks, queue: &wgpu::Queue) {
        self.timer.update(delta_time);
        let current_time = self.timer.time();

        let mut skipped_frames = 0;
        // find the latest frame that is ready for display
        // this might be the currently pending frame, or any of the frames after it (shouldn't happen often I think)
        while let Some((timing, ref frame)) = self.pending_frame {
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
            let next_frame = match self.video_decoder.read_frame() {
                Ok(frame) => frame,
                Err(err) => {
                    error!("Error reading frame: {}. Stopping playback", err);
                    None
                }
            };

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
                self.video_texture.write_data_nv12(queue, frame);
                // the loop will not enter again, so the pending frame will now be displayed
            } else {
                skipped_frames += 1;
                // if the next frame is also ready for display, then we should skip the pending frame
            }

            if next_frame.is_none() {
                info!("No more frames, stopping playback");
            }

            self.pending_frame = next_frame;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.pending_frame.is_none()
    }

    pub fn render(&self, pass: &mut RenderPass) {
        pass.run(RenderRequestBuilder::new().build(
            RenderProgramWithArguments::Movie {
                vertices: VertexSource::VertexData {
                    vertices: &[
                        MovieVertex {
                            coords: Vec4::new(0.0, 0.0, 0.0, 0.0),
                        },
                        MovieVertex {
                            coords: Vec4::new(1.0, 0.0, 1.0, 0.0),
                        },
                        MovieVertex {
                            coords: Vec4::new(0.0, 1.0, 0.0, 1.0),
                        },
                        MovieVertex {
                            coords: Vec4::new(1.0, 1.0, 1.0, 1.0),
                        },
                    ],
                },
                texture_luma: self.video_texture.get_y_source(),
                texture_chroma: self.video_texture.get_uv_source(),
                // TODO: the coordinate systems are probably all whack
                transform: Mat4::from_scale(Vec3::new(2.0, 2.0, -1.0))
                    * Mat4::from_translation(Vec3::new(-0.5, -0.5, 0.0)),
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
