use tracing::warn;

use crate::layout::text_layouter::TextLayouter;

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedCommand {
    /// Just your regular character (or a @U command)
    Char(char),
    /// @+
    EnableLipsync,
    /// @-
    DisableLipsync,
    /// @/
    VoiceWait,
    /// @b
    RubiContent(String),
    /// @<
    RubiBaseStart,
    /// @>
    RubiBaseEnd,
    /// @a
    SetFade(i32),
    /// @c
    SetColor(i32),
    /// @e
    NoFinalClickWait,
    /// @k
    ClickWait,
    /// @o
    VoiceVolume(i32),
    /// @r
    Newline,
    /// @s
    TextSpeed(i32),
    /// @t
    StartParallel,
    /// @v
    Voice(String),
    /// @w
    Wait(i32),
    /// @x
    VoiceSync(i32),
    /// @y
    Sync,
    /// @z
    FontScale(i32),
    /// @|
    CompleteSection,
    /// @[
    InstantTextStart,
    /// @]
    InstantTextEnd,
    /// @{
    BoldTextStart,
    /// @}
    BoldTextEnd,
}

pub struct MessageTextParser<'a> {
    message: &'a str,
}

impl<'a> MessageTextParser<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }

    pub fn parse_into<L: TextLayouter>(self, layouter: &mut L) {
        layouter.on_message_start();

        for command in self {
            match command {
                ParsedCommand::Char(codepoint) => layouter.on_char(codepoint),
                ParsedCommand::EnableLipsync => layouter.on_lipsync_enabled(),
                ParsedCommand::DisableLipsync => layouter.on_lipsync_disabled(),
                ParsedCommand::VoiceWait => layouter.on_voice_wait(),
                ParsedCommand::RubiContent(text) => layouter.on_rubi_content(text),
                ParsedCommand::RubiBaseStart => layouter.on_rubi_base_start(),
                ParsedCommand::RubiBaseEnd => layouter.on_rubi_base_end(),
                ParsedCommand::SetFade(fade) => layouter.on_set_fade(fade),
                ParsedCommand::SetColor(color) => layouter.on_set_color(color),
                ParsedCommand::NoFinalClickWait => layouter.on_auto_click(),
                ParsedCommand::ClickWait => layouter.on_click_wait(),
                ParsedCommand::VoiceVolume(volume) => layouter.on_set_voice_volume(volume),
                ParsedCommand::Newline => layouter.on_newline(),
                ParsedCommand::TextSpeed(speed) => layouter.on_set_draw_speed(speed),
                ParsedCommand::StartParallel => layouter.on_start_parallel(),
                ParsedCommand::Voice(voice) => layouter.on_voice(voice),
                ParsedCommand::Wait(wait) => layouter.on_wait(wait),
                ParsedCommand::VoiceSync(target_instant) => layouter.on_voice_sync(target_instant),
                ParsedCommand::Sync => layouter.on_sync(),
                ParsedCommand::FontScale(scale) => layouter.on_set_font_scale(scale),
                ParsedCommand::CompleteSection => layouter.on_section(),
                ParsedCommand::InstantTextStart => layouter.on_instant_start(),
                ParsedCommand::InstantTextEnd => layouter.on_instant_end(),
                ParsedCommand::BoldTextStart => layouter.on_bold_start(),
                ParsedCommand::BoldTextEnd => layouter.on_bold_end(),
            }
        }

        layouter.on_message_end();
    }

    fn read_string_argument(&mut self) -> String {
        let Some(end) = self.message.find('.') else {
            return "".to_string();
        };
        let argument = &self.message[..end];
        self.message = &self.message[end + 1..];

        // NOTE: the original MessageTextParser applies a fixup to the returned string argument
        // we have already applied the fixup when decoding the message though, so this should not be necessary
        // TODO: We __might__ want to move the fixup decoding to the same place the game does it, potentially saving some debugging time later...

        argument.to_string()
    }

    fn read_int_argument(&mut self) -> i32 {
        let Some(end) = self.message.find('.') else {
            return -1;
        };
        let mut argument = self.message[..end].chars();
        self.message = &self.message[end + 1..];

        let Some(head_char) = argument.next() else {
            return -1;
        };

        if head_char == '$' {
            let mut accumulator = 0;
            for c in argument {
                accumulator = accumulator * 16 + c.to_digit(16).unwrap_or(0);
            }

            accumulator as i32
        } else {
            let mut accumulator = head_char.to_digit(10).unwrap_or(0);
            for c in argument {
                accumulator = accumulator * 10 + c.to_digit(10).unwrap_or(0);
            }

            accumulator as i32
        }
    }
}

impl Iterator for MessageTextParser<'_> {
    type Item = ParsedCommand;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.message.chars();
        let first_char = chars.next()?;

        if first_char != '@' {
            self.message = chars.as_str();
            return Some(ParsedCommand::Char(first_char));
        }

        let Some(second_char) = chars.next() else {
            // trailing `@` without a command is treated as-if the line has ended, without emitting anything
            return None;
        };
        self.message = chars.as_str();

        Some(match second_char {
            '+' => ParsedCommand::EnableLipsync,
            '-' => ParsedCommand::DisableLipsync,
            '/' => ParsedCommand::VoiceWait,
            'b' => ParsedCommand::RubiContent(self.read_string_argument()),
            '<' => ParsedCommand::RubiBaseStart,
            '>' => ParsedCommand::RubiBaseEnd,
            'a' => ParsedCommand::SetFade(self.read_int_argument()),
            'c' => ParsedCommand::SetColor(self.read_int_argument()),
            'e' => ParsedCommand::NoFinalClickWait,
            'k' => ParsedCommand::ClickWait,
            'o' => ParsedCommand::VoiceVolume(self.read_int_argument()),
            'r' => ParsedCommand::Newline,
            's' => ParsedCommand::TextSpeed(self.read_int_argument()),
            't' => ParsedCommand::StartParallel,
            'v' => ParsedCommand::Voice(self.read_string_argument().to_owned()),
            'w' => ParsedCommand::Wait(self.read_int_argument()),
            'x' => ParsedCommand::VoiceSync(self.read_int_argument()),
            'y' => ParsedCommand::Sync,
            'z' => ParsedCommand::FontScale(self.read_int_argument()),
            '|' => ParsedCommand::CompleteSection,
            '[' => ParsedCommand::InstantTextStart,
            ']' => ParsedCommand::InstantTextEnd,
            '{' => ParsedCommand::BoldTextStart,
            '}' => ParsedCommand::BoldTextEnd,
            // NB: the original engine does not do any validation here. We have to if we want to continue using the char type
            'U' => ParsedCommand::Char(
                char::from_u32(self.read_int_argument() as u32).expect("Invalid char code"),
            ),
            c => {
                warn!("Unknown layouter command: {}", c);
                ParsedCommand::Char(c)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(message: &str) -> Vec<ParsedCommand> {
        MessageTextParser::new(message).collect()
    }

    #[test]
    fn test_hello() {
        let message = "Hello";
        let commands = parse(message);

        assert_eq!(
            commands,
            vec![
                ParsedCommand::Char('H'),
                ParsedCommand::Char('e'),
                ParsedCommand::Char('l'),
                ParsedCommand::Char('l'),
                ParsedCommand::Char('o')
            ]
        );
    }

    #[test]
    fn test_furigana() {
        let message = "@bかな.@<漢字@>";
        let commands = parse(message);

        assert_eq!(
            commands,
            vec![
                ParsedCommand::RubiContent("かな".to_owned()),
                ParsedCommand::RubiBaseStart,
                ParsedCommand::Char('漢'),
                ParsedCommand::Char('字'),
                ParsedCommand::RubiBaseEnd,
            ]
        );
    }

    #[test]
    fn test_color() {
        let message = "@c940.@rHello@c.";
        let commands = parse(message);

        assert_eq!(
            commands,
            vec![
                ParsedCommand::SetColor(940),
                ParsedCommand::Newline,
                ParsedCommand::Char('H'),
                ParsedCommand::Char('e'),
                ParsedCommand::Char('l'),
                ParsedCommand::Char('l'),
                ParsedCommand::Char('o'),
                ParsedCommand::SetColor(-1),
            ]
        );
    }

    #[test]
    fn test_wait() {
        let message = "Hello@w400.@rWorld";
        let commands = parse(message);

        assert_eq!(
            commands,
            vec![
                ParsedCommand::Char('H'),
                ParsedCommand::Char('e'),
                ParsedCommand::Char('l'),
                ParsedCommand::Char('l'),
                ParsedCommand::Char('o'),
                ParsedCommand::Wait(400),
                ParsedCommand::Newline,
                ParsedCommand::Char('W'),
                ParsedCommand::Char('o'),
                ParsedCommand::Char('r'),
                ParsedCommand::Char('l'),
                ParsedCommand::Char('d'),
            ]
        );
    }

    #[test]
    fn test_unfinished_command() {
        let message = "Hello@r@";
        let commands = parse(message);

        assert_eq!(
            commands,
            vec![
                ParsedCommand::Char('H'),
                ParsedCommand::Char('e'),
                ParsedCommand::Char('l'),
                ParsedCommand::Char('l'),
                ParsedCommand::Char('o'),
                ParsedCommand::Newline,
            ]
        );
    }

    #[test]
    fn test_unknown_command() {
        let message = "Hello@!";
        let commands = parse(message);

        assert_eq!(
            commands,
            vec![
                ParsedCommand::Char('H'),
                ParsedCommand::Char('e'),
                ParsedCommand::Char('l'),
                ParsedCommand::Char('l'),
                ParsedCommand::Char('o'),
                ParsedCommand::Char('!'),
            ]
        );
    }

    #[test]
    fn test_real1() {
        // TODO: why is this not fixed up again?
        let message = "@r@v00/awase6042_o.@|@y｢｢@c900.@[謹啓､謹ﾝで申ｼ上げﾙ｡@k@v00/awase6043_o.どﾁﾗﾓ破ﾗﾚﾃｲﾅｲﾓﾉﾄ知ﾘ給ｴ@]@c.｣｣";
        let commands = parse(message);

        assert_eq!(
            commands,
            vec![
                ParsedCommand::Newline,
                ParsedCommand::Voice("00/awase6042_o".to_owned()),
                ParsedCommand::CompleteSection,
                ParsedCommand::Sync,
                ParsedCommand::Char('｢'),
                ParsedCommand::Char('｢'),
                ParsedCommand::SetColor(900),
                ParsedCommand::InstantTextStart,
                ParsedCommand::Char('謹'),
                ParsedCommand::Char('啓'),
                ParsedCommand::Char('､'),
                ParsedCommand::Char('謹'),
                ParsedCommand::Char('ﾝ'),
                ParsedCommand::Char('で'),
                ParsedCommand::Char('申'),
                ParsedCommand::Char('ｼ'),
                ParsedCommand::Char('上'),
                ParsedCommand::Char('げ'),
                ParsedCommand::Char('ﾙ'),
                ParsedCommand::Char('｡'),
                ParsedCommand::ClickWait,
                ParsedCommand::Voice("00/awase6043_o".to_owned()),
                ParsedCommand::Char('ど'),
                ParsedCommand::Char('ﾁ'),
                ParsedCommand::Char('ﾗ'),
                ParsedCommand::Char('ﾓ'),
                ParsedCommand::Char('破'),
                ParsedCommand::Char('ﾗ'),
                ParsedCommand::Char('ﾚ'),
                ParsedCommand::Char('ﾃ'),
                ParsedCommand::Char('ｲ'),
                ParsedCommand::Char('ﾅ'),
                ParsedCommand::Char('ｲ'),
                ParsedCommand::Char('ﾓ'),
                ParsedCommand::Char('ﾉ'),
                ParsedCommand::Char('ﾄ'),
                ParsedCommand::Char('知'),
                ParsedCommand::Char('ﾘ'),
                ParsedCommand::Char('給'),
                ParsedCommand::Char('ｴ'),
                ParsedCommand::InstantTextEnd,
                ParsedCommand::SetColor(-1),
                ParsedCommand::Char('｣'),
                ParsedCommand::Char('｣'),
            ]
        );
    }

    #[test]
    fn test_real2() {
        let message = "めぐみん@r@vMGM_00310.「いえ、紅魔族の辞書に反省の文字は……@x324.@|@yああーっ！　ごめんなさい、ごめんなさい！　その怪しい手の動きはやめてください！」";
        let commands = parse(message);

        assert_eq!(
            commands,
            vec![
                ParsedCommand::Char('め'),
                ParsedCommand::Char('ぐ'),
                ParsedCommand::Char('み'),
                ParsedCommand::Char('ん'),
                ParsedCommand::Newline,
                ParsedCommand::Voice("MGM_00310".to_owned()),
                ParsedCommand::Char('「'),
                ParsedCommand::Char('い'),
                ParsedCommand::Char('え'),
                ParsedCommand::Char('、'),
                ParsedCommand::Char('紅'),
                ParsedCommand::Char('魔'),
                ParsedCommand::Char('族'),
                ParsedCommand::Char('の'),
                ParsedCommand::Char('辞'),
                ParsedCommand::Char('書'),
                ParsedCommand::Char('に'),
                ParsedCommand::Char('反'),
                ParsedCommand::Char('省'),
                ParsedCommand::Char('の'),
                ParsedCommand::Char('文'),
                ParsedCommand::Char('字'),
                ParsedCommand::Char('は'),
                ParsedCommand::Char('…'),
                ParsedCommand::Char('…'),
                ParsedCommand::VoiceSync(324),
                ParsedCommand::CompleteSection,
                ParsedCommand::Sync,
                ParsedCommand::Char('あ'),
                ParsedCommand::Char('あ'),
                ParsedCommand::Char('ー'),
                ParsedCommand::Char('っ'),
                ParsedCommand::Char('！'),
                ParsedCommand::Char('　'),
                ParsedCommand::Char('ご'),
                ParsedCommand::Char('め'),
                ParsedCommand::Char('ん'),
                ParsedCommand::Char('な'),
                ParsedCommand::Char('さ'),
                ParsedCommand::Char('い'),
                ParsedCommand::Char('、'),
                ParsedCommand::Char('ご'),
                ParsedCommand::Char('め'),
                ParsedCommand::Char('ん'),
                ParsedCommand::Char('な'),
                ParsedCommand::Char('さ'),
                ParsedCommand::Char('い'),
                ParsedCommand::Char('！'),
                ParsedCommand::Char('　'),
                ParsedCommand::Char('そ'),
                ParsedCommand::Char('の'),
                ParsedCommand::Char('怪'),
                ParsedCommand::Char('し'),
                ParsedCommand::Char('い'),
                ParsedCommand::Char('手'),
                ParsedCommand::Char('の'),
                ParsedCommand::Char('動'),
                ParsedCommand::Char('き'),
                ParsedCommand::Char('は'),
                ParsedCommand::Char('や'),
                ParsedCommand::Char('め'),
                ParsedCommand::Char('て'),
                ParsedCommand::Char('く'),
                ParsedCommand::Char('だ'),
                ParsedCommand::Char('さ'),
                ParsedCommand::Char('い'),
                ParsedCommand::Char('！'),
                ParsedCommand::Char('」'),
            ]
        );
    }
}
