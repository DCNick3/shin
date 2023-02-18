use anyhow::{Context, Result};
use std::io::Write;
use std::process::Stdio;
use tracing::debug;

enum Output {
    Unparsed(Option<std::process::ChildStdout>),
    Parsed(y4m::Decoder<std::process::ChildStdout>),
}

impl Output {
    fn ensure_parsed(&mut self) -> Result<()> {
        match self {
            Output::Unparsed(stdout) => {
                debug!("Starting to parse ffmpeg output");
                let stdout = stdout.take().unwrap();
                let decoder = y4m::Decoder::new(stdout).context("Error creating y4m decoder")?;
                *self = Output::Parsed(decoder);
                Ok(())
            }
            Output::Parsed(_) => Ok(()),
        }
    }

    pub fn read_frame(&mut self) -> Result<Option<y4m::Frame>> {
        self.ensure_parsed()?;
        match self {
            Output::Parsed(decoder) => match decoder.read_frame() {
                Ok(frame) => Ok(Some(frame)),
                Err(y4m::Error::EOF) => Ok(None),
                Err(e) => Err(e).context("Could not read frame from ffmpeg"),
            },
            _ => unreachable!(),
        }
    }

    pub fn info(&mut self) -> Result<FrameInfo> {
        self.ensure_parsed()?;
        match self {
            Output::Parsed(dec) => Ok(FrameInfo {
                colorspace: dec.get_colorspace(),
                width: dec.get_width(),
                height: dec.get_height(),
            }),
            _ => unreachable!(),
        }
    }
}

pub struct FrameInfo {
    pub colorspace: y4m::Colorspace,
    pub width: usize,
    pub height: usize,
}

/// Decodes h264 Annex B format to YUV420P (or, possibly, some other frame format).
///
/// Currently implemented as a pipe to the ffmpeg binary, other options are possible in the future.
pub struct H264Decoder {
    #[allow(dead_code)]
    process: std::process::Child,
    stdin: Option<std::process::ChildStdin>,
    stdout: Output,
    // stderr: std::process::ChildStderr,
}

impl H264Decoder {
    pub fn new() -> Result<Self> {
        // TODO: use a more robust way to find the ffmpeg binary
        let ffmpeg = which::which("ffmpeg").context("Could not locate ffmpeg binary")?;

        let mut process = std::process::Command::new(ffmpeg)
            .arg("-loglevel")
            .arg("debug")
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
            .stderr(Stdio::inherit())
            .spawn()
            .context("Could not spawn ffmpeg process")?;

        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();
        // let stderr = process.stderr.take().unwrap();

        let stdin = Some(stdin);
        let stdout = Output::Unparsed(Some(stdout));

        Ok(Self {
            process,
            stdin,
            stdout,
            // stderr,
        })
    }

    pub fn push_packet(&mut self, packet: &[u8]) -> Result<()> {
        debug!("Pushing packet to ffmpeg ({} bytes)", packet.len());
        self.stdin.as_mut().unwrap().write_all(packet)?;
        Ok(())
    }

    pub fn read_frame(&mut self) -> Result<Option<y4m::Frame>> {
        debug!("Reading frame from ffmpeg");
        self.stdout.read_frame()
    }

    pub fn mark_eof(&mut self) {
        if self.stdin.take().is_some() {
            debug!("Closing stdin")
        }
    }

    pub fn info(&mut self) -> Result<FrameInfo> {
        debug!("Reading frame info from ffmpeg");
        self.stdout.info()
    }
}

impl Drop for H264Decoder {
    fn drop(&mut self) {
        self.process.kill().unwrap();
    }
}
