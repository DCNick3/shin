use std::{
    fs::File,
    io::{BufReader, ErrorKind, Read},
};

use glam::Vec2;
use minicbor::Decode;
use num_traits::FromPrimitive as _;

use crate::{
    format::font::FontInfo,
    layout::{
        message_text_layouter::{
            commands::Command, LayoutParams, LineInfo, MessageLayerLayouter,
            MessageTextLayouterDefaults,
        },
        MessageTextParser,
    },
    vm::command::types::{MessageTextLayout, MessageboxType},
};

#[derive(Decode)]
struct Entry {
    // Note: the point we dump the message from stores the message in a non-fixed up form
    // however, our layouter expects a fixed up form.
    #[n(0)]
    message: String,
    #[n(1)]
    messagebox_style: u32,
    #[n(2)]
    text_alignment: u32,
    #[n(3)]
    message_id: i32,
    #[n(4)]
    snapshot: String,
}

struct Fonts {
    bold_font: FontInfo,
    normal_font: FontInfo,
}

fn read_fonts() -> Fonts {
    fn read_font(path: &str) -> FontInfo {
        let font = File::open(path).unwrap();
        let mut font = BufReader::new(font);
        shin_core::format::font::read_font_metrics(&mut font).unwrap()
    }

    let normal = read_font("test_assets/newrodin-medium.fnt");
    let bold = read_font("test_assets/newrodin-bold.fnt");

    Fonts {
        bold_font: bold,
        normal_font: normal,
    }
}

fn make_snapshot(
    fonts: &Fonts,
    text_alignment: MessageTextLayout,
    messagebox_type: MessageboxType,
    text: &str,
) -> (Vec<Command>, Vec<LineInfo>, Vec2) {
    let normal = &fonts.normal_font;
    let bold = &fonts.bold_font;

    // layout params used by the MessageLayer
    let layout_params = LayoutParams {
        layout_width: 1500.0,
        text_alignment,
        line_spacing: 0.0,
        another_line_height: 0.0,
        line_height3: 4.0,
        rubi_size: 20.0,
        text_size: 50.0,
        base_font_horizontal_scale: 0.9697,
        follow_kinsoku_shori_rules: true,
        always_leave_space_for_rubi: true,
        perform_soft_breaks: true,
    };
    let defaults = MessageTextLayouterDefaults {
        color: 999,
        draw_speed: 80,
        fade: 200,
    };

    let mut layouter =
        MessageLayerLayouter::new(normal, bold, messagebox_type, layout_params, defaults);
    let parser = MessageTextParser::new(text);
    parser.parse_into(&mut layouter);

    (
        layouter.layouter.commands,
        layouter.layouter.lines,
        layouter.layouter.size,
    )
}

fn check_layout_dump(path: &str) {
    let fonts = read_fonts();

    let mut decoder = lz4_flex::frame::FrameDecoder::new(std::fs::File::open(path).unwrap());
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

        let Entry {
            message,
            messagebox_style,
            text_alignment,
            message_id,
            snapshot: expected_snapshot,
        } = minicbor::decode::<Entry>(&buf).unwrap();
        let message = shin_core::format::text::decode_string_fixup(&message);

        let messagebox_style = MessageboxType::from_u32(messagebox_style).unwrap();
        let text_alignment = MessageTextLayout::from_u32(text_alignment).unwrap();

        let snapshot = make_snapshot(&fonts, text_alignment, messagebox_style, &message);
        let snapshot = format!("{:#?}", snapshot);

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
fn layout_dump_ep1() {
    let path = "test_assets/layout_dumps/ep1.cbor";
    check_layout_dump(path);
}
