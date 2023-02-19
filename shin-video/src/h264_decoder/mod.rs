mod y4m;

use std::iter::Peekable;
pub use y4m::{BitsPerSample, Colorspace, Frame, FrameSize, PlaneSize};

use crate::mp4::Mp4TrackReader;
use crate::Mp4BitstreamConverter;
use anyhow::{bail, Context, Result};
use futures_lite::io::BufReader;
use futures_lite::{AsyncBufReadExt, AsyncWriteExt};
use std::process::Stdio;
use tracing::{debug, error, trace};

/// Decodes h264 Annex B format to YUV420P (or, possibly, some other frame format).
///
/// Currently implemented as a pipe to the ffmpeg binary, other options are possible in the future.
pub struct H264Decoder {
    process: async_process::Child,
    frame_receiver: Peekable<std::sync::mpsc::IntoIter<Frame>>,
    // frame_timing_receiver:
    #[allow(unused)]
    frame_sender_task: bevy_tasks::Task<()>,
    #[allow(unused)]
    packet_sender_task: bevy_tasks::Task<()>,
    #[allow(unused)]
    stderr_task: bevy_tasks::Task<()>,

    frame_info: Option<FrameSize>,
}

// const FFMPEG_LOG_LEVEL: &str = "debug";
const FFMPEG_LOG_LEVEL: &str = "info";

impl H264Decoder {
    pub fn new(track: Mp4TrackReader) -> Result<Self> {
        // TODO: use a more robust way to find the ffmpeg binary
        let ffmpeg = which::which("ffmpeg").context("Could not locate ffmpeg binary")?;

        let mut process = async_process::Command::new(ffmpeg)
            .arg("-loglevel")
            .arg(FFMPEG_LOG_LEVEL)
            .arg("-f")
            .arg("h264")
            .arg("-flags")
            .arg("low_delay")
            .arg("-analyzeduration")
            .arg("0")
            .arg("-probesize")
            .arg("32")
            .arg("-i")
            .arg("pipe:0")
            .arg("-f")
            .arg("yuv4mpegpipe")
            .arg("pipe:1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Could not spawn ffmpeg process")?;

        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();
        let stderr = process.stderr.take().unwrap();

        // let stdin = Some(stdin);
        // let stdout = Output::Unparsed(Some(stdout));

        let (frame_sender, frame_receiver) = std::sync::mpsc::sync_channel(1);

        let frame_sender_task = bevy_tasks::IoTaskPool::get().spawn(async move {
            let mut decoder = match y4m::Decoder::new(stdout).await {
                Ok(r) => r,
                Err(e) => {
                    error!(
                        "Error creating y4m decoder: {}. The stream will be closed",
                        e
                    );
                    return;
                }
            };
            loop {
                match decoder.read_frame().await {
                    Ok(frame) => {
                        trace!("Sending frame to game");
                        frame_sender.send(frame).unwrap();
                    }
                    Err(y4m::Error::EOF) => {
                        debug!("EOF from ffmpeg, stopping sending to game");
                        break;
                    }
                    Err(e) => {
                        error!("Error reading frame from ffmpeg: {}", e);
                        break;
                    }
                }
            }
        });

        let packet_sender_task = bevy_tasks::IoTaskPool::get().spawn(async move {
            let mut track = track;
            let mut buffer = Vec::new();
            let mut stdin = stdin;

            let mut bitstream_converter: Mp4BitstreamConverter =
                track.get_mp4_track_info(Mp4BitstreamConverter::for_mp4_track);

            loop {
                match track.next_sample() {
                    Ok(Some(sample)) => {
                        bitstream_converter.convert_packet(&sample.bytes, &mut buffer);
                        trace!("Sending sample to ffmpeg ({} bytes)", buffer.len());
                        match stdin.write_all(&buffer).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Error writing sample to ffmpeg: {}", e);
                                break;
                            }
                        }
                    }
                    Ok(None) => {
                        debug!("EOF from mp4, stopping sending to ffmpeg");
                        break;
                    }
                    Err(e) => {
                        error!("Error reading sample from mp4: {}", e);
                        break;
                    }
                }
            }
        });

        let stderr_task = bevy_tasks::IoTaskPool::get().spawn(async move {
            let mut stderr = BufReader::new(stderr);
            let mut buffer = String::new();

            loop {
                match stderr.read_line(&mut buffer).await {
                    Ok(0) => {
                        debug!("EOF from ffmpeg stderr, stopping reading");
                        break;
                    }
                    Ok(_) => {
                        trace!("ffmpeg: {}", buffer);
                        buffer.clear();
                    }
                    Err(e) => {
                        error!("Error reading ffmpeg stderr: {}", e);
                        break;
                    }
                }
            }
        });

        let frame_receiver = frame_receiver.into_iter().peekable();

        Ok(Self {
            process,
            frame_receiver,
            frame_sender_task,
            packet_sender_task,
            stderr_task,
            frame_info: None,
        })
    }

    pub fn read_frame(&mut self) -> Result<Option<Frame>> {
        trace!("Reading frame from ffmpeg...");
        match self.frame_receiver.next() {
            Some(frame) => {
                self.frame_info = Some(*frame.size());
                Ok(Some(frame))
            }
            None => Ok(None),
        }
    }

    pub fn info(&mut self) -> Result<FrameSize> {
        debug!("Reading frame info from ffmpeg");
        match self.frame_info {
            Some(info) => Ok(info),
            None => match self.frame_receiver.peek() {
                Some(frame) => {
                    self.frame_info = Some(*frame.size());
                    Ok(*frame.size())
                }
                None => {
                    bail!("No frames available, don't know the format")
                }
            },
        }
    }
}

impl Drop for H264Decoder {
    fn drop(&mut self) {
        self.process.kill().unwrap();
    }
}
