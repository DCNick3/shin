use cgmath::Vector3;

#[derive(Debug, Clone, PartialEq)]
pub enum LayouterCommand {
    /// Just your regular character (or a @U command)
    Char(char),
    /// @+
    EnableLipsync,
    /// @-
    DisableLipsync,
    /// @b
    Furigana(String),
    /// @<
    FuriganaStart,
    /// @>
    FuriganaEnd,
    /// @a
    SetFade(f32),
    /// @c
    SetColor(Option<Vector3<f32>>),
    /// @e
    AutoClick,
    /// @k
    WaitClick,
    /// @o
    VoiceVolume(f32),
    /// @r
    Newline,
    /// @s
    TextSpeed(f32),
    /// @t
    SimultaneousStart,
    /// @v
    Voice(String),
    /// @w
    Wait(f32),
    /// @y
    Sync,
    /// @z
    FontSize(f32),
    /// @|
    Signal,
    /// @[
    InstantTextStart,
    /// @]
    InstantTextEnd,
    /// @{
    BoldTextStart,
    /// @}
    BoldTextEnd,
}

pub struct LayouterParser<'a> {
    message: &'a str,
}

impl<'a> LayouterParser<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }

    fn read_argument(&mut self) -> &'a str {
        let end = self
            .message
            .find('.')
            .expect("Could not find the end of the argument");
        let argument = &self.message[..end];
        self.message = &self.message[end + 1..];
        argument
    }

    fn read_float_argument(&mut self, min: u32, max: u32, scale: f32) -> f32 {
        let argument = self.read_argument();
        let value = argument.parse::<u32>().expect("Could not parse argument");
        let value = value.clamp(min.min(max), max.max(min));
        // if min max are backwards - reverse the value
        let value = if min > max { max - value } else { value };
        value as f32 / scale
    }

    fn read_color_argument(&mut self) -> Option<Vector3<f32>> {
        let argument = self.read_argument();
        if argument.is_empty() {
            None
        } else {
            let mut chars = argument.chars();
            let r = chars.next().unwrap().to_digit(10).unwrap() as f32 / 9.0;
            let g = chars.next().unwrap().to_digit(10).unwrap() as f32 / 9.0;
            let b = chars.next().unwrap().to_digit(10).unwrap() as f32 / 9.0;
            assert!(chars.next().is_none());
            Some(Vector3::new(r, g, b))
        }
    }
}

impl Iterator for LayouterParser<'_> {
    type Item = LayouterCommand;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: make this parsing fallible

        if self.message.is_empty() {
            return None;
        }

        let mut chars = self.message.chars();
        let first_char = chars.next().unwrap();

        if first_char != '@' {
            self.message = chars.as_str();
            return Some(LayouterCommand::Char(first_char));
        }

        let second_char = chars.next().unwrap();
        self.message = chars.as_str();

        Some(match second_char {
            '+' => LayouterCommand::EnableLipsync,
            '-' => LayouterCommand::DisableLipsync,
            'b' => LayouterCommand::Furigana(self.read_argument().to_owned()),
            '<' => LayouterCommand::FuriganaStart,
            '>' => LayouterCommand::FuriganaEnd,
            'a' => LayouterCommand::SetFade(self.read_float_argument(0, u32::MAX, 1000.0)),
            'c' => LayouterCommand::SetColor(self.read_color_argument()),
            'e' => LayouterCommand::AutoClick,
            'k' => LayouterCommand::WaitClick,
            'o' => LayouterCommand::VoiceVolume(self.read_float_argument(0, 100, 100.0)),
            'r' => LayouterCommand::Newline,
            's' => LayouterCommand::TextSpeed(self.read_float_argument(100, 0, 40000.0)),
            't' => LayouterCommand::SimultaneousStart,
            'v' => LayouterCommand::Voice(self.read_argument().to_owned()),
            'w' => LayouterCommand::Wait(self.read_float_argument(0, u32::MAX, 100.0)),
            'y' => LayouterCommand::Sync,
            'z' => LayouterCommand::FontSize(self.read_float_argument(10, 200, 100.0)),
            '|' => LayouterCommand::Signal,
            '[' => LayouterCommand::InstantTextStart,
            ']' => LayouterCommand::InstantTextEnd,
            '{' => LayouterCommand::BoldTextStart,
            '}' => LayouterCommand::BoldTextEnd,
            'U' => todo!("@U layouter command parsing"),
            _ => panic!("Unknown layouter command: {}", second_char),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(message: &str) -> Vec<LayouterCommand> {
        LayouterParser::new(message).collect()
    }

    #[test]
    fn test_hello() {
        let message = "Hello";
        let commands = parse(message);

        assert_eq!(
            commands,
            vec![
                LayouterCommand::Char('H'),
                LayouterCommand::Char('e'),
                LayouterCommand::Char('l'),
                LayouterCommand::Char('l'),
                LayouterCommand::Char('o')
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
                LayouterCommand::Furigana("かな".to_owned()),
                LayouterCommand::FuriganaStart,
                LayouterCommand::Char('漢'),
                LayouterCommand::Char('字'),
                LayouterCommand::FuriganaEnd,
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
                LayouterCommand::SetColor(Some(Vector3::new(1.0, 4.0 / 9.0, 0.0))),
                LayouterCommand::Newline,
                LayouterCommand::Char('H'),
                LayouterCommand::Char('e'),
                LayouterCommand::Char('l'),
                LayouterCommand::Char('l'),
                LayouterCommand::Char('o'),
                LayouterCommand::SetColor(None),
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
                LayouterCommand::Char('H'),
                LayouterCommand::Char('e'),
                LayouterCommand::Char('l'),
                LayouterCommand::Char('l'),
                LayouterCommand::Char('o'),
                LayouterCommand::Wait(4.0),
                LayouterCommand::Newline,
                LayouterCommand::Char('W'),
                LayouterCommand::Char('o'),
                LayouterCommand::Char('r'),
                LayouterCommand::Char('l'),
                LayouterCommand::Char('d'),
            ]
        );
    }

    #[test]
    fn test_real1() {
        let message = "@r@v00/awase6042_o.@|@y｢｢@c900.@[謹啓､謹ﾝで申ｼ上げﾙ｡@k@v00/awase6043_o.どﾁﾗﾓ破ﾗﾚﾃｲﾅｲﾓﾉﾄ知ﾘ給ｴ@]@c.｣｣";
        let commands = parse(message);

        assert_eq!(
            commands,
            vec![
                LayouterCommand::Newline,
                LayouterCommand::Voice("00/awase6042_o".to_owned()),
                LayouterCommand::Signal,
                LayouterCommand::Sync,
                LayouterCommand::Char('｢'),
                LayouterCommand::Char('｢'),
                LayouterCommand::SetColor(Some(Vector3::new(1.0, 0.0, 0.0))),
                LayouterCommand::InstantTextStart,
                LayouterCommand::Char('謹'),
                LayouterCommand::Char('啓'),
                LayouterCommand::Char('､'),
                LayouterCommand::Char('謹'),
                LayouterCommand::Char('ﾝ'),
                LayouterCommand::Char('で'),
                LayouterCommand::Char('申'),
                LayouterCommand::Char('ｼ'),
                LayouterCommand::Char('上'),
                LayouterCommand::Char('げ'),
                LayouterCommand::Char('ﾙ'),
                LayouterCommand::Char('｡'),
                LayouterCommand::WaitClick,
                LayouterCommand::Voice("00/awase6043_o".to_owned()),
                LayouterCommand::Char('ど'),
                LayouterCommand::Char('ﾁ'),
                LayouterCommand::Char('ﾗ'),
                LayouterCommand::Char('ﾓ'),
                LayouterCommand::Char('破'),
                LayouterCommand::Char('ﾗ'),
                LayouterCommand::Char('ﾚ'),
                LayouterCommand::Char('ﾃ'),
                LayouterCommand::Char('ｲ'),
                LayouterCommand::Char('ﾅ'),
                LayouterCommand::Char('ｲ'),
                LayouterCommand::Char('ﾓ'),
                LayouterCommand::Char('ﾉ'),
                LayouterCommand::Char('ﾄ'),
                LayouterCommand::Char('知'),
                LayouterCommand::Char('ﾘ'),
                LayouterCommand::Char('給'),
                LayouterCommand::Char('ｴ'),
                LayouterCommand::InstantTextEnd,
                LayouterCommand::SetColor(None),
                LayouterCommand::Char('｣'),
                LayouterCommand::Char('｣'),
            ]
        );
    }
}
