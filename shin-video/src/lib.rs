//! Glue together mp4 demuxing, h264 and aac decoding and `shin-render` APIs to implement video playback in `shin`.

mod audio;
mod h264_decoder;
pub mod mp4;
mod mp4_bitstream_converter;
mod texture;
mod timer;
mod video_player;

pub use texture::VideoFrameTexture;
pub use video_player::VideoPlayer;
