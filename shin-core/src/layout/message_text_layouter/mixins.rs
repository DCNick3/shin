use crate::{
    layout::message_text_layouter::{
        commands::Command, font::FontMetrics, MessageTextLayouterImpl, MessageTextLayouterMixin,
    },
    vm::command::types::MessageboxType,
};

pub struct NoMixin;

impl<Font: FontMetrics> MessageTextLayouterMixin<Font> for NoMixin {
    fn on_char(&mut self, layouter: &mut MessageTextLayouterImpl<Font>, codepoint: char) {
        layouter.on_char(codepoint);
    }

    fn on_newline(&mut self, layouter: &mut MessageTextLayouterImpl<Font>) {
        layouter.on_newline(self);
    }

    fn on_voice(&mut self, layouter: &mut MessageTextLayouterImpl<Font>, voice_path: String) {
        layouter.on_voice(voice_path);
    }

    fn finalize_up_to(
        &mut self,
        layouter: &mut MessageTextLayouterImpl<Font>,
        finalize_index: usize,
        is_hard_break: bool,
    ) {
        layouter.finalize_up_to(finalize_index, is_hard_break);
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum QuotationState {
    Ignored = -1,
    #[default]
    Uninit = 0,
    Open = 1,
}

pub struct MessageLayerLayouterMixin {
    pub messagebox_type: MessageboxType,
    pub line_index: u32,
    pub quotation_state: QuotationState,
    pub quotation_opener: char,
    pub quotation_level: i32,
    pub quotation_indent: f32,
}

impl MessageLayerLayouterMixin {
    const CHARACTER_NAME_FONT_SIZE: i32 = 90;
    const CHARACTER_NAME_WIDTH: f32 = 360.0;

    pub fn new(messagebox_type: MessageboxType) -> Self {
        Self {
            messagebox_type,
            line_index: 0,
            quotation_state: QuotationState::Uninit,
            quotation_opener: '\0',
            quotation_level: 0,
            quotation_indent: 0.0,
        }
    }
}

impl<Font: FontMetrics> MessageTextLayouterMixin<Font> for MessageLayerLayouterMixin {
    fn on_char(&mut self, layouter: &mut MessageTextLayouterImpl<Font>, codepoint: char) {
        if self.line_index == 0 {
            // the first line if special: it's used for character names
            if self.messagebox_type == MessageboxType::Novel {
                // don't render the character name in novel mode
                return;
            }
            layouter.font_scale = super::parse_font_scale(Self::CHARACTER_NAME_FONT_SIZE);
            layouter.is_inside_instant_text = true;

            layouter.on_char(codepoint)
        } else {
            layouter.on_char(codepoint)
        }
    }

    fn on_newline(&mut self, layouter: &mut MessageTextLayouterImpl<Font>) {
        if self.line_index == 0 {
            if self.messagebox_type != MessageboxType::Novel {
                let old_leave_space_for_rubi = layouter.params.always_leave_space_for_rubi;
                layouter.params.always_leave_space_for_rubi = true;
                layouter.font_scale = super::parse_font_scale(Self::CHARACTER_NAME_FONT_SIZE);
                // NB: not calling through a mixin here, because we __don't__ want quotation handling in character names
                layouter.finalize_up_to(layouter.commands.len(), true);

                let line_info = &mut layouter.lines[0];
                if line_info.width > 0.0 {
                    let x_offset = if line_info.width < Self::CHARACTER_NAME_WIDTH {
                        let offset = (Self::CHARACTER_NAME_WIDTH - line_info.width) / 2.0;
                        line_info.width = Self::CHARACTER_NAME_WIDTH;
                        offset
                    } else {
                        0.0
                    };

                    for cmd in &mut layouter.commands {
                        let Command::Char(char) = cmd else {
                            continue;
                        };
                        char.position.x += x_offset;

                        if !char.is_rubi {
                            char.position.y -= layouter.params.rubi_size;
                        }
                    }
                }

                layouter.font_scale = layouter.default_font_scale;
                layouter.position.y -=
                    layouter.params.line_height3 + layouter.params.rubi_size - 20.0;
                layouter.is_inside_instant_text = false;
                layouter.params.always_leave_space_for_rubi = old_leave_space_for_rubi;
            }
        } else {
            layouter.on_newline(self);
            if self.quotation_state == QuotationState::Ignored {
                self.quotation_state = QuotationState::Uninit;
            }
        }
        self.line_index += 1;
    }

    fn on_voice(&mut self, layouter: &mut MessageTextLayouterImpl<Font>, voice_path: String) {
        layouter.on_voice(voice_path)
    }

    fn finalize_up_to(
        &mut self,
        layouter: &mut MessageTextLayouterImpl<Font>,
        finalize_index: usize,
        is_hard_break: bool,
    ) {
        // preserve the `finalized_command_count` var, as it would be updated by the call to `finalize_up_to`
        let finalized_command_count = layouter.finalized_command_count;
        layouter.finalize_up_to(finalize_index, is_hard_break);
        match self.quotation_state {
            QuotationState::Uninit => {
                for cmd in &layouter.commands[finalized_command_count..finalize_index] {
                    let Command::Char(char) = cmd else {
                        continue;
                    };
                    if char.is_rubi {
                        continue;
                    }
                    if char.codepoint != '「' && char.codepoint != '（' && char.codepoint != '『'
                    {
                        self.quotation_state = QuotationState::Ignored;
                        return;
                    }
                    layouter.params.layout_width += self.quotation_indent;
                    self.quotation_state = QuotationState::Open;
                    self.quotation_indent = char.width;
                    self.quotation_opener = char.codepoint;
                    self.quotation_level = 0;
                    layouter.params.layout_width -= self.quotation_indent;
                    break;
                }
            }
            QuotationState::Open => {
                for cmd in &mut layouter.commands[finalized_command_count..finalize_index] {
                    if let Command::Char(char) = cmd {
                        char.position.x += self.quotation_indent;
                    };
                }
            }
            QuotationState::Ignored => {
                return;
            }
        }

        for cmd in &layouter.commands[finalized_command_count..finalize_index] {
            let Command::Char(char) = cmd else {
                continue;
            };
            if char.is_rubi {
                continue;
            }
            if char.codepoint == self.quotation_opener {
                self.quotation_level += 1;
            } else if char.codepoint as u32 == self.quotation_opener as u32 + 1 {
                self.quotation_level -= 1;
            }
        }

        if self.quotation_level < 1 {
            layouter.params.layout_width += self.quotation_indent;
            self.quotation_indent = 0.0;
            self.quotation_state = QuotationState::Uninit;
        }
    }
}
