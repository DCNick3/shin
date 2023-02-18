use futures_lite::io::BufReader;
use futures_lite::{AsyncBufReadExt, AsyncRead, AsyncReadExt};
use std::fmt;
use std::io;
use std::io::Read;
use std::num;
use std::str;

const FILE_MAGICK: &[u8] = b"YUV4MPEG2 ";
const FRAME_MAGICK: &[u8] = b"FRAME";
const TERMINATOR: u8 = 0x0A;
const FIELD_SEP: u8 = b' ';
const RATIO_SEP: u8 = b':';

/// Both encoding and decoding errors.
#[derive(Debug)]
pub enum Error {
    /// End of the file. Technically not an error, but it's easier to process
    /// that way.
    EOF,
    /// Bad input parameters provided.
    BadInput,
    /// Unknown colorspace (possibly just unimplemented).
    UnknownColorspace,
    /// Error while parsing the file/frame header.
    // TODO(Kagami): Better granularity of parse errors.
    ParseError(ParseError),
    /// Error while reading/writing the file.
    IoError(io::Error),
    /// Out of memory (limits exceeded).
    OutOfMemory,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::EOF => None,
            Error::BadInput => None,
            Error::UnknownColorspace => None,
            Error::ParseError(ref err) => Some(err),
            Error::IoError(ref err) => Some(err),
            Error::OutOfMemory => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::EOF => write!(f, "End of file"),
            Error::BadInput => write!(f, "Bad input parameters provided"),
            Error::UnknownColorspace => write!(f, "Bad input parameters provided"),
            Error::ParseError(ref err) => err.fmt(f),
            Error::IoError(ref err) => err.fmt(f),
            Error::OutOfMemory => write!(f, "Out of memory (limits exceeded)"),
        }
    }
}

/// Granular ParseError Definiations
pub enum ParseError {
    /// Error reading y4m header
    InvalidY4M,
    /// Error parsing int
    Int,
    /// Error parsing UTF8
    Utf8,
    /// General Parsing Error
    General,
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ParseError::InvalidY4M => None,
            ParseError::Int => None,
            ParseError::Utf8 => None,
            ParseError::General => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidY4M => write!(f, "Error parsing y4m header"),
            ParseError::Int => write!(f, "Error parsing Int"),
            ParseError::Utf8 => write!(f, "Error parsing UTF8"),
            ParseError::General => write!(f, "General parsing error"),
        }
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidY4M => write!(f, "Error parsing y4m header"),
            ParseError::Int => write!(f, "Error parsing Int"),
            ParseError::Utf8 => write!(f, "Error parsing UTF8"),
            ParseError::General => write!(f, "General parsing error"),
        }
    }
}

macro_rules! parse_error {
    ($p:expr) => {
        return Err(Error::ParseError($p))
    };
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        match err.kind() {
            io::ErrorKind::UnexpectedEof => Error::EOF,
            _ => Error::IoError(err),
        }
    }
}

impl From<num::ParseIntError> for Error {
    fn from(_: num::ParseIntError) -> Error {
        Error::ParseError(ParseError::Int)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(_: str::Utf8Error) -> Error {
        Error::ParseError(ParseError::Utf8)
    }
}

trait EnhancedRead {
    fn read_until(&mut self, ch: u8, buf: &mut [u8]) -> Result<usize, Error>;
}

impl<R: Read> EnhancedRead for R {
    // Current implementation does one `read` call per byte. This might be a
    // bit slow for long headers but it simplifies things: we don't need to
    // check whether start of the next frame is already read and so on.
    fn read_until(&mut self, ch: u8, buf: &mut [u8]) -> Result<usize, Error> {
        let mut collected = 0;
        while collected < buf.len() {
            let chunk_size = self.read(&mut buf[collected..=collected])?;
            if chunk_size == 0 {
                return Err(Error::EOF);
            }
            if buf[collected] == ch {
                return Ok(collected);
            }
            collected += chunk_size;
        }
        parse_error!(ParseError::General)
    }
}

fn parse_bytes(buf: &[u8]) -> Result<usize, Error> {
    // A bit kludgy but seems like there is no other way.
    Ok(str::from_utf8(buf)?.parse()?)
}

/// A newtype wrapper around Vec<u8> to ensure validity as a vendor extension.
#[derive(Debug, Clone)]
pub struct VendorExtensionString(Vec<u8>);

impl VendorExtensionString {
    /// Create a new vendor extension string.
    ///
    /// For example, setting to `b"COLORRANGE=FULL"` sets the interpretation of
    /// the YUV values to cover the full range (rather a limited "studio swing"
    /// range).
    ///
    /// The argument `x_option` must not contain a space (b' ') character,
    /// otherwise [Error::BadInput] is returned.
    pub fn new(value: Vec<u8>) -> Result<VendorExtensionString, Error> {
        if value.contains(&b' ') {
            return Err(Error::BadInput);
        }
        Ok(VendorExtensionString(value))
    }
    /// Get the vendor extension string.
    pub fn value(&self) -> &[u8] {
        self.0.as_slice()
    }
}

/// Simple ratio structure since stdlib lacks one.
#[derive(Debug, Clone, Copy)]
pub struct Ratio {
    /// Numerator.
    pub num: usize,
    /// Denominator.
    pub den: usize,
}

impl Ratio {
    /// Create a new ratio.
    pub fn new(num: usize, den: usize) -> Ratio {
        Ratio { num, den }
    }

    /// Parse a ratio from a byte slice.
    pub fn parse(value: &[u8]) -> Result<Ratio, Error> {
        let parts: Vec<_> = value.splitn(2, |&b| b == RATIO_SEP).collect();
        if parts.len() != 2 {
            parse_error!(ParseError::General)
        }
        let num = parse_bytes(parts[0])?;
        let den = parse_bytes(parts[1])?;
        Ok(Ratio::new(num, den))
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.num, self.den)
    }
}

/// Colorspace (color model/pixel format). Only subset of them is supported.
///
/// From libavformat/yuv4mpegenc.c:
///
/// > yuv4mpeg can only handle yuv444p, yuv422p, yuv420p, yuv411p and gray8
/// pixel formats. And using 'strict -1' also yuv444p9, yuv422p9, yuv420p9,
/// yuv444p10, yuv422p10, yuv420p10, yuv444p12, yuv422p12, yuv420p12,
/// yuv444p14, yuv422p14, yuv420p14, yuv444p16, yuv422p16, yuv420p16, gray9,
/// gray10, gray12 and gray16 pixel formats.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Colorspace {
    /// Grayscale only, 8-bit.
    Cmono,
    /// Grayscale only, 12-bit.
    Cmono12,
    /// 4:2:0 with coincident chroma planes, 8-bit.
    C420,
    /// 4:2:0 with coincident chroma planes, 10-bit.
    C420p10,
    /// 4:2:0 with coincident chroma planes, 12-bit.
    C420p12,
    /// 4:2:0 with biaxially-displaced chroma planes, 8-bit.
    C420jpeg,
    /// 4:2:0 with vertically-displaced chroma planes, 8-bit.
    C420paldv,
    /// Found in some files. Same as `C420`.
    C420mpeg2,
    /// 4:2:2, 8-bit.
    C422,
    /// 4:2:2, 10-bit.
    C422p10,
    /// 4:2:2, 12-bit.
    C422p12,
    /// 4:4:4, 8-bit.
    C444,
    /// 4:4:4, 10-bit.
    C444p10,
    /// 4:4:4, 12-bit.
    C444p12,
}

impl Colorspace {
    /// Return the bit depth per sample
    #[inline]
    pub fn get_bit_depth(self) -> usize {
        match self {
            Colorspace::Cmono
            | Colorspace::C420
            | Colorspace::C422
            | Colorspace::C444
            | Colorspace::C420jpeg
            | Colorspace::C420paldv
            | Colorspace::C420mpeg2 => 8,
            Colorspace::C420p10 | Colorspace::C422p10 | Colorspace::C444p10 => 10,
            Colorspace::Cmono12
            | Colorspace::C420p12
            | Colorspace::C422p12
            | Colorspace::C444p12 => 12,
        }
    }

    /// Return the number of bytes in a sample
    #[inline]
    pub fn get_bytes_per_sample(self) -> usize {
        if self.get_bit_depth() <= 8 {
            1
        } else {
            2
        }
    }
}

fn get_plane_sizes(width: usize, height: usize, colorspace: Colorspace) -> (usize, usize, usize) {
    let y_plane_size = width * height * colorspace.get_bytes_per_sample();

    let c420_chroma_size =
        ((width + 1) / 2) * ((height + 1) / 2) * colorspace.get_bytes_per_sample();
    let c422_chroma_size = ((width + 1) / 2) * height * colorspace.get_bytes_per_sample();

    let c420_sizes = (y_plane_size, c420_chroma_size, c420_chroma_size);
    let c422_sizes = (y_plane_size, c422_chroma_size, c422_chroma_size);
    let c444_sizes = (y_plane_size, y_plane_size, y_plane_size);

    match colorspace {
        Colorspace::Cmono | Colorspace::Cmono12 => (y_plane_size, 0, 0),
        Colorspace::C420
        | Colorspace::C420p10
        | Colorspace::C420p12
        | Colorspace::C420jpeg
        | Colorspace::C420paldv
        | Colorspace::C420mpeg2 => c420_sizes,
        Colorspace::C422 | Colorspace::C422p10 | Colorspace::C422p12 => c422_sizes,
        Colorspace::C444 | Colorspace::C444p10 | Colorspace::C444p12 => c444_sizes,
    }
}

/// Limits on the resources `Decoder` is allowed to use.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum allowed size of frame buffer, default is 1 GiB.
    pub bytes: usize,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            bytes: 1024 * 1024 * 1024,
        }
    }
}

/// YUV4MPEG2 decoder.
pub struct Decoder<R: AsyncRead> {
    reader: BufReader<R>,
    params_buf: Vec<u8>,
    frame_buf: Vec<u8>,
    raw_params: Vec<u8>,
    width: usize,
    height: usize,
    framerate: Ratio,
    pixel_aspect: Ratio,
    colorspace: Colorspace,
    y_len: usize,
    u_len: usize,
}

impl<R: AsyncRead + Unpin> Decoder<R> {
    /// Create a new decoder instance.
    pub async fn new(reader: R) -> Result<Decoder<R>, Error> {
        Decoder::new_with_limits(reader, Limits::default()).await
    }

    /// Create a new decoder instance with custom limits.
    pub async fn new_with_limits(reader: R, limits: Limits) -> Result<Decoder<R>, Error> {
        let mut raw_params = Vec::new();
        let mut buf_reader = BufReader::new(reader);
        let end_params_pos = buf_reader.read_until(TERMINATOR, &mut raw_params).await?;
        if end_params_pos < FILE_MAGICK.len() || !raw_params.starts_with(FILE_MAGICK) {
            parse_error!(ParseError::InvalidY4M)
        }
        let mut width = 0;
        let mut height = 0;
        // Framerate is actually required per spec, but let's be a bit more
        // permissive as per ffmpeg behavior.
        let mut framerate = Ratio::new(25, 1);
        let mut pixel_aspect = Ratio::new(1, 1);
        let mut colorspace = None;
        // We shouldn't convert it to string because encoding is unspecified.
        for param in raw_params.split(|&b| b == FIELD_SEP) {
            if param.is_empty() {
                continue;
            }
            let (name, value) = (param[0], &param[1..]);
            // TODO(Kagami): interlacing, comment.
            match name {
                b'W' => width = parse_bytes(value)?,
                b'H' => height = parse_bytes(value)?,
                b'F' => framerate = Ratio::parse(value)?,
                b'A' => pixel_aspect = Ratio::parse(value)?,
                b'C' => {
                    colorspace = match value {
                        b"mono" => Some(Colorspace::Cmono),
                        b"mono12" => Some(Colorspace::Cmono12),
                        b"420" => Some(Colorspace::C420),
                        b"420p10" => Some(Colorspace::C420p10),
                        b"420p12" => Some(Colorspace::C420p12),
                        b"422" => Some(Colorspace::C422),
                        b"422p10" => Some(Colorspace::C422p10),
                        b"422p12" => Some(Colorspace::C422p12),
                        b"444" => Some(Colorspace::C444),
                        b"444p10" => Some(Colorspace::C444p10),
                        b"444p12" => Some(Colorspace::C444p12),
                        b"420jpeg" => Some(Colorspace::C420jpeg),
                        b"420paldv" => Some(Colorspace::C420paldv),
                        b"420mpeg2" => Some(Colorspace::C420mpeg2),
                        _ => return Err(Error::UnknownColorspace),
                    }
                }
                _ => {}
            }
        }
        let colorspace = colorspace.unwrap_or(Colorspace::C420);
        if width == 0 || height == 0 {
            parse_error!(ParseError::General)
        }
        let (y_len, u_len, v_len) = get_plane_sizes(width, height, colorspace);
        let frame_size = y_len + u_len + v_len;
        if frame_size > limits.bytes {
            return Err(Error::OutOfMemory);
        }
        let frame_buf = vec![0; frame_size];
        Ok(Decoder {
            reader: buf_reader,
            params_buf: Vec::new(),
            frame_buf,
            raw_params,
            width,
            height,
            framerate,
            pixel_aspect,
            colorspace,
            y_len,
            u_len,
        })
    }

    /// Iterate over frames. End of input is indicated by `Error::EOF`.
    pub async fn read_frame(&mut self) -> Result<Frame, Error> {
        self.params_buf.clear();
        self.reader
            .read_until(TERMINATOR, &mut self.params_buf)
            .await?;

        if self.params_buf.is_empty() {
            return Err(Error::EOF);
        }

        self.params_buf.resize(self.params_buf.len() - 1, 0); // remove the terminator
        if self.params_buf.len() < FRAME_MAGICK.len() || !self.params_buf.starts_with(FRAME_MAGICK)
        {
            parse_error!(ParseError::InvalidY4M)
        }
        // We don't parse frame params currently but user has access to them.
        let start_params_pos = FRAME_MAGICK.len();
        let raw_params = if self.params_buf.len() - start_params_pos > 0 {
            // Check for extra space.
            if dbg!(self.params_buf[start_params_pos]) != FIELD_SEP {
                parse_error!(ParseError::InvalidY4M)
            }
            Some(self.params_buf[start_params_pos + 1..].to_owned())
        } else {
            None
        };
        self.reader.read_exact(&mut self.frame_buf).await?;
        Ok(Frame::new(
            [
                self.frame_buf[0..self.y_len].to_vec(),
                self.frame_buf[self.y_len..self.y_len + self.u_len].to_vec(),
                self.frame_buf[self.y_len + self.u_len..].to_vec(),
            ],
            raw_params,
            FrameInfo {
                width: self.width,
                height: self.height,
                colorspace: self.colorspace,
            },
        ))
    }

    /// Return file width.
    #[inline]
    pub fn get_width(&self) -> usize {
        self.width
    }
    /// Return file height.
    #[inline]
    pub fn get_height(&self) -> usize {
        self.height
    }
    /// Return file framerate.
    #[inline]
    pub fn get_framerate(&self) -> Ratio {
        self.framerate
    }
    /// Return file pixel aspect.
    #[inline]
    pub fn get_pixel_aspect(&self) -> Ratio {
        self.pixel_aspect
    }
    /// Return file colorspace.
    ///
    /// **NOTE:** normally all .y4m should have colorspace param, but there are
    /// files encoded without that tag and it's unclear what should we do in
    /// that case. Currently C420 is implied by default as per ffmpeg behavior.
    #[inline]
    pub fn get_colorspace(&self) -> Colorspace {
        self.colorspace
    }
    /// Return file raw parameters.
    #[inline]
    pub fn get_raw_params(&self) -> &[u8] {
        &self.raw_params
    }
    /// Return the bit depth per sample
    #[inline]
    pub fn get_bit_depth(&self) -> usize {
        self.colorspace.get_bit_depth()
    }
    /// Return the number of bytes in a sample
    #[inline]
    pub fn get_bytes_per_sample(&self) -> usize {
        self.colorspace.get_bytes_per_sample()
    }
}

// TODO: store sizes separate per plane
#[derive(Debug, Copy, Clone)]
pub struct FrameInfo {
    pub colorspace: Colorspace,
    pub width: usize,
    pub height: usize,
}

/// A single frame.
#[derive(Debug)]
pub struct Frame {
    planes: [Vec<u8>; 3],
    raw_params: Option<Vec<u8>>,
    info: FrameInfo,
}

impl Frame {
    /// Create a new frame with optional parameters.
    /// No heap allocations are made.
    pub fn new(planes: [Vec<u8>; 3], raw_params: Option<Vec<u8>>, info: FrameInfo) -> Frame {
        Frame {
            planes,
            raw_params,
            info,
        }
    }

    /// Return Y (first) plane.
    #[inline]
    pub fn get_y_plane(&self) -> &[u8] {
        self.planes[0].as_ref()
    }
    /// Return U (second) plane. Empty in case of grayscale.
    #[inline]
    pub fn get_u_plane(&self) -> &[u8] {
        self.planes[1].as_ref()
    }
    /// Return V (third) plane. Empty in case of grayscale.
    #[inline]
    pub fn get_v_plane(&self) -> &[u8] {
        self.planes[2].as_ref()
    }
    /// Return frame raw parameters if any.
    #[inline]
    pub fn get_raw_params(&self) -> Option<&[u8]> {
        self.raw_params.as_ref().map(|v| &v[..])
    }

    #[inline]
    pub fn info(&self) -> &FrameInfo {
        &self.info
    }
}

/// Create a new decoder instance. Alias for `Decoder::new`.
pub async fn decode<R: AsyncRead + Unpin>(reader: R) -> Result<Decoder<R>, Error> {
    Decoder::new(reader).await
}
