mod dumps;
mod snapshots;

use std::{fs::File, io::BufReader, sync::LazyLock};

use glam::Vec2;

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

pub struct TestFonts {
    bold_font: FontInfo,
    normal_font: FontInfo,
}

pub fn read_fonts() -> TestFonts {
    fn read_font(path: &str) -> FontInfo {
        let font = File::open(path).unwrap();
        let mut font = BufReader::new(font);
        shin_core::format::font::read_font_metrics(&mut font).unwrap()
    }

    let normal = read_font("test_assets/newrodin-medium.fnt");
    let bold = read_font("test_assets/newrodin-bold.fnt");
    // let system = read_font("test_assets/system.fnt");

    TestFonts {
        bold_font: bold,
        normal_font: normal,
    }
}

pub fn make_snapshot(
    text_alignment: MessageTextLayout,
    messagebox_type: MessageboxType,
    text: &str,
) -> (Vec<Command>, Vec<LineInfo>, Vec2) {
    // share fonts between invocations in the same process
    static FONTS: LazyLock<TestFonts> = LazyLock::new(|| read_fonts());

    let normal = &FONTS.normal_font;
    let bold = &FONTS.bold_font;

    // layout params used by the MessageLayer
    let layout_params = LayoutParams {
        layout_width: 1500.0,
        text_alignment,
        line_padding_above: 0.0,
        line_padding_below: 0.0,
        line_padding_between: 4.0,
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

    layouter.finish()
}
