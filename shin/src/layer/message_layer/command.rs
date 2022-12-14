use shin_core::layout::CharCommand;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Char(CharCommand),
}
