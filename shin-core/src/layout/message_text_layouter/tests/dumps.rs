//! Test on full-chapter layout dumps
//!
//! These are less fine-grained, but much more comprehensive.
//!
//! To ease debugging of a dump-based test failed, it's easier to extract the failing message and make a snapshot test for it.

use std::io::{ErrorKind, Read as _};

use num_traits::FromPrimitive as _;

use crate::vm::command::types::{MessageTextLayout, MessageboxType};

/// Contains copies of the snapshotted types, but supporting cbor encoding
///
/// Field names should be kept in sync with the real ones
mod model {
    use std::fmt;

    use minicbor::{Decode, Encode};

    #[derive(Clone, Copy, PartialEq, Encode, Decode)]
    #[repr(C)]
    pub struct Vec2 {
        #[n(0)]
        pub x: f32,
        #[n(1)]
        pub y: f32,
    }

    impl fmt::Display for Vec2 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if let Some(p) = f.precision() {
                write!(f, "[{:.*}, {:.*}]", p, self.x, p, self.y)
            } else {
                write!(f, "[{}, {}]", self.x, self.y)
            }
        }
    }

    impl fmt::Debug for Vec2 {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt.debug_tuple(stringify!(Vec2))
                .field(&self.x)
                .field(&self.y)
                .finish()
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode)]
    #[repr(transparent)]
    pub struct UnormColor(#[n(0)] pub u32);

    #[derive(Debug, PartialEq, Copy, Clone, Encode, Decode)]
    pub enum CharFontType {
        #[n(0)]
        Regular,
        #[n(1)]
        Bold,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    pub struct Char {
        #[n(0)]
        pub time: f32,
        #[n(1)]
        pub line_index: usize,
        #[n(2)]
        pub font: CharFontType,
        #[n(3)]
        pub codepoint: char,
        #[n(4)]
        pub is_rubi: bool,
        #[n(5)]
        pub cant_be_at_line_start: bool,
        #[n(6)]
        pub cant_be_at_line_end: bool,
        #[n(7)]
        pub has_rubi: bool,
        #[n(8)]
        pub width: f32,
        #[n(9)]
        pub height: f32,
        #[n(10)]
        pub position: Vec2,
        #[n(11)]
        pub horizontal_scale: f32,
        #[n(12)]
        pub scale: f32,
        #[n(13)]
        pub color: UnormColor,
        #[n(14)]
        pub fade: f32,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    pub struct Section {
        #[n(0)]
        pub time: f32,
        #[n(1)]
        pub line_index: usize,
        #[n(2)]
        pub index: u32,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    pub struct Sync {
        #[n(0)]
        pub time: f32,
        #[n(1)]
        pub line_index: usize,
        #[n(2)]
        pub index: u32,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    pub struct Voice {
        #[n(0)]
        pub time: f32,
        #[n(1)]
        pub line_index: usize,
        #[n(2)]
        pub filename: String,
        #[n(3)]
        pub volume: f32,
        #[n(4)]
        pub lipsync_enabled: bool,
        #[n(5)]
        pub segment_duration: i32,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    pub struct VoiceSync {
        #[n(0)]
        pub time: f32,
        #[n(1)]
        pub line_index: usize,
        #[n(2)]
        pub segment_start: i32,
        #[n(3)]
        pub segment_duration: i32,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    pub struct VoiceWait {
        #[n(0)]
        pub time: f32,
        #[n(1)]
        pub line_index: usize,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    pub struct Wait {
        #[n(0)]
        pub time: f32,
        #[n(1)]
        pub line_index: usize,
        #[n(2)]
        pub is_last_wait: bool,
        #[n(3)]
        pub is_auto_click: bool,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    pub enum Command {
        #[n(0)]
        Char(#[n(0)] Char),
        #[n(1)]
        Section(#[n(0)] Section),
        #[n(2)]
        Sync(#[n(0)] Sync),
        #[n(3)]
        Voice(#[n(0)] Voice),
        #[n(4)]
        VoiceSync(#[n(0)] VoiceSync),
        #[n(5)]
        VoiceWait(#[n(0)] VoiceWait),
        #[n(6)]
        Wait(#[n(0)] Wait),
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    pub struct LineInfo {
        #[n(0)]
        pub width: f32,
        #[n(1)]
        pub y_position: f32,
        #[n(2)]
        pub line_height: f32,
        #[n(3)]
        pub baseline_ascent: f32,
        #[n(4)]
        pub rubi_height: f32,
    }

    pub type Snapshot = (Vec<Command>, Vec<LineInfo>, Vec2);

    #[derive(Decode)]
    pub struct Entry {
        // Note: the point we dump the message from stores the message in a non-fixed up form
        // however, our layouter expects a fixed up form.
        #[n(0)]
        pub message: String,
        #[n(1)]
        pub messagebox_style: u32,
        #[n(2)]
        pub text_alignment: u32,
        #[n(3)]
        pub message_id: i32,
        #[n(4)]
        pub snapshot: Snapshot,
    }
}

fn check_layout_dump(path: &str) {
    let decoder = std::fs::File::open(path).unwrap();
    let mut decoder = zstd::Decoder::new(decoder).unwrap();
    let mut buf = vec![];

    let mut fail_count = 0;
    let mut success_count = 0;

    loop {
        let mut len_buf = [0; 4];

        match decoder.read_exact(&mut len_buf) {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                panic!("Failed to read length: {}", e);
            }
        }
        let len = u32::from_le_bytes(len_buf);
        buf.resize(len as usize, 0);

        decoder.read_exact(&mut buf).unwrap();

        let model::Entry {
            message,
            messagebox_style,
            text_alignment,
            message_id,
            snapshot: expected_snapshot,
        } = minicbor::decode(&buf).unwrap();
        let message = shin_core::format::text::decode_string_fixup(&message);

        let messagebox_style = MessageboxType::from_u32(messagebox_style).unwrap();
        let text_alignment = MessageTextLayout::from_u32(text_alignment).unwrap();

        let expected_snapshot = format!("{:#?}", expected_snapshot);

        let snapshot = super::make_snapshot(text_alignment, messagebox_style, &message);
        let snapshot = format!("{:#?}", snapshot);

        // NOTE: we are comparing debug representations of different types. This allows us to keep the CBOR encoding logic self-contained to this module
        if snapshot == expected_snapshot {
            success_count += 1;
        } else {
            fail_count += 1;

            if fail_count == 15 {
                println!("Too many failures, no diff will be printed");
            }
            if fail_count < 15 {
                let diff = similar_asserts::SimpleDiff::from_str(
                    &snapshot,
                    &expected_snapshot,
                    "Computed snapshot",
                    "Ground Truth",
                );

                println!(
                    "Snapshot mismatch for message_id = {}: {:?}",
                    message_id, message
                );
                println!(
                    "messagebox_style={:?}, text_alignment={:?}",
                    messagebox_style, text_alignment
                );
                println!("Ground truth: {:?}", expected_snapshot);
                println!("{}", diff);
                println!()
            } else {
                println!(
                    "Snapshot mismatch for message_id = {}: {:?}",
                    message_id, message
                );
            }
        }
    }

    println!(
        "Success: {}/{} ({:.01}%)",
        success_count,
        fail_count + success_count,
        success_count as f64 / (fail_count + success_count) as f64 * 100.0
    );

    if fail_count > 0 {
        panic!("Some snapshots failed to match");
    }
}

#[test]
fn ep1() {
    let path = "test_assets/layout_dumps/ep1.cbor.zst";
    check_layout_dump(path);
}

#[test]
fn ep2() {
    let path = "test_assets/layout_dumps/ep2.cbor.zst";
    check_layout_dump(path);
}

#[test]
fn ep3() {
    let path = "test_assets/layout_dumps/ep3.cbor.zst";
    check_layout_dump(path);
}

#[test]
fn ep4() {
    let path = "test_assets/layout_dumps/ep4.cbor.zst";
    check_layout_dump(path);
}
