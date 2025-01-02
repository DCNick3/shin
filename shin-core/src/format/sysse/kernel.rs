//! This module implements the core logic of decoding sysse file
//!
//! Sysse audio is encoded using a codec that is derivative of Sony's ADPCM scheme used on the PlayStation (see https://jsgroth.dev/blog/posts/ps1-spu-part-1/#adpcm).
//!
//! The main difference is that the block does not include "flags", which are replaced by another two samples, making it a 30 samples per block codec.

use binrw::{BinRead, BinWrite};
use proc_bitfield::bitfield;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BlockHeader(pub u8) : Debug {
        pub shift: u8 @ 0..4,
        pub filter: usize @ 4..7,
    }
}

#[derive(Copy, Clone)]
struct Fir2 {
    ir_1: i32,
    ir_2: i32,
}

impl Fir2 {
    #[inline]
    pub fn evaluate_history(&self, x: History) -> i32 {
        self.ir_1 * x.history_1 + self.ir_2 * x.history_2
    }
}

const DECODER_FIR_TABLE: [Fir2; 5] = [
    Fir2 { ir_1: 0, ir_2: 0 },
    Fir2 { ir_1: 60, ir_2: 0 },
    Fir2 {
        ir_1: 115,
        ir_2: -52,
    },
    Fir2 {
        ir_1: 98,
        ir_2: -55,
    },
    Fir2 {
        ir_1: 122,
        ir_2: -60,
    },
];

#[derive(Copy, Clone)]
struct History {
    history_1: i32,
    history_2: i32,
}

impl History {
    #[inline]
    pub fn new() -> Self {
        Self {
            history_1: 0,
            history_2: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, new_sample: i32) {
        self.history_2 = self.history_1;
        self.history_1 = new_sample;
    }
}

#[allow(non_camel_case_types)]
struct i4(i32);

impl i4 {
    #[inline]
    pub fn from_raw_bits(bits: u8) -> Self {
        assert!(bits < 16);

        Self((bits as i32) << 28 >> 28)
    }

    #[inline]
    pub fn extract_two_from_byte(byte: u8) -> (i4, i4) {
        let a = i4::from_raw_bits(byte & 0xf);
        let b = i4::from_raw_bits(byte >> 4);

        (a, b)
    }
}

impl From<i4> for i32 {
    #[inline]
    fn from(i: i4) -> i32 {
        i.0
    }
}

struct SampleDecoder {
    sample_scale_bits: u8,
    history: History,
    fir: Fir2,
}

impl SampleDecoder {
    pub fn new(sample_scale_bits: u8, history: History, fir: Fir2) -> Self {
        Self {
            sample_scale_bits,
            history,
            fir,
        }
    }

    /// Decodes a single sample from a half-byte.
    #[inline]
    pub fn decode_sample(&mut self, sample_data: i4) -> i16 {
        let sample_residual: i32 = sample_data.into();

        let mut predicted_value = self.fir.evaluate_history(self.history);
        predicted_value += 32; // apply bias
        if predicted_value < 0 {
            predicted_value += 63; // round towards zero
        }
        let predicted_value = predicted_value >> 6;

        let sample = predicted_value + (sample_residual << self.sample_scale_bits);

        let sample = sample.clamp(i16::MIN as i32, i16::MAX as i32);

        self.history.push(sample);

        sample as i16
    }

    fn finish(self) -> History {
        self.history
    }
}

const BLOCK_SIZE_BYTES: usize = 16;
pub const BLOCK_SIZE: usize = 30;

fn decode_block(history: &mut History, block_data: [u8; BLOCK_SIZE_BYTES]) -> [i16; BLOCK_SIZE] {
    let header = BlockHeader(block_data[0]);
    let shift = header.shift();
    let filter = header.filter();

    let mut sample_decoder = SampleDecoder::new(shift, *history, DECODER_FIR_TABLE[filter]);

    let mut samples = [0; BLOCK_SIZE];
    assert_eq!(samples.len(), (BLOCK_SIZE_BYTES - 1) * 2);

    for (i, byte) in (0..).step_by(2).zip(&block_data[1..]) {
        let (sample1, sample2) = i4::extract_two_from_byte(*byte);

        samples[i] = sample_decoder.decode_sample(sample1);
        samples[i + 1] = sample_decoder.decode_sample(sample2);
    }

    *history = sample_decoder.finish();

    samples
}

#[inline]
fn convert_sample(sample: i16) -> f32 {
    (sample as f32 / i16::MAX as f32).clamp(-1.0, 1.0)
}

pub trait DecoderKernel {
    fn decode_block(&mut self) -> Option<[(f32, f32); BLOCK_SIZE]>;
}

struct MonoDecoderKernel {
    data: std::vec::IntoIter<[u8; 16]>,
    block_index: usize,
    history: History,
}

impl MonoDecoderKernel {
    fn new(data: Vec<u8>) -> Self {
        let data = bytemuck::cast_vec(data);

        Self {
            data: data.into_iter(),
            block_index: 0,
            history: History::new(),
        }
    }
}

impl DecoderKernel for MonoDecoderKernel {
    fn decode_block(&mut self) -> Option<[(f32, f32); BLOCK_SIZE]> {
        let undecoded_block = self.data.next()?;

        let decoded_block = decode_block(&mut self.history, undecoded_block);
        Some(decoded_block.map(|v| (convert_sample(v), convert_sample(v))))
    }
}

struct StereoDecoderKernel {
    data: std::vec::IntoIter<[[u8; 16]; 2]>,
    block_index: usize,
    history_left: History,
    history_right: History,
}

impl StereoDecoderKernel {
    fn new(data: Vec<u8>) -> Self {
        let data = bytemuck::cast_vec(data);

        Self {
            data: data.into_iter(),
            block_index: 0,
            history_left: History::new(),
            history_right: History::new(),
        }
    }
}

impl DecoderKernel for StereoDecoderKernel {
    fn decode_block(&mut self) -> Option<[(f32, f32); BLOCK_SIZE]> {
        let [undecoded_block_left, undecoded_block_right] = self.data.next()?;

        let decoded_block_left = decode_block(&mut self.history_left, undecoded_block_left);
        let decoded_block_right = decode_block(&mut self.history_right, undecoded_block_right);

        let mut result = [(0.0, 0.0); BLOCK_SIZE];
        for (i, (l, r)) in (0..).zip(decoded_block_left.into_iter().zip(decoded_block_right)) {
            result[i] = (convert_sample(l), convert_sample(r));
        }

        Some(result)
    }
}

#[derive(Copy, Clone, BinRead, BinWrite)]
#[brw(repr = u16)]
pub enum ChannelCount {
    Mono = 1,
    Stereo = 2,
}

pub enum EitherDecoderKernel {
    Mono(MonoDecoderKernel),
    Stereo(StereoDecoderKernel),
}

impl EitherDecoderKernel {
    pub fn new(data: Vec<u8>, channel_count: ChannelCount) -> Self {
        match channel_count {
            ChannelCount::Mono => Self::Mono(MonoDecoderKernel::new(data)),
            ChannelCount::Stereo => Self::Stereo(StereoDecoderKernel::new(data)),
        }
    }
}

impl DecoderKernel for EitherDecoderKernel {
    fn decode_block(&mut self) -> Option<[(f32, f32); BLOCK_SIZE]> {
        match self {
            Self::Mono(inner) => inner.decode_block(),
            Self::Stereo(inner) => inner.decode_block(),
        }
    }
}
