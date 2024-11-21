use enum_map::Enum;
pub use winit::keyboard::KeyCode;

// for the gamepad we emulate nintendo switch controller (because that's what I was reversing =))

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Enum)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

impl GamepadAxis {
    pub fn from_gilrs(axis: gilrs::Axis) -> Option<Self> {
        use gilrs::Axis::*;
        Some(match axis {
            LeftStickX => GamepadAxis::LeftStickX,
            LeftStickY => GamepadAxis::LeftStickY,
            RightStickX => GamepadAxis::RightStickX,
            RightStickY => GamepadAxis::RightStickY,
            LeftZ => GamepadAxis::LeftTrigger,
            RightZ => GamepadAxis::RightTrigger,
            _ => return None,
        })
    }
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Enum)]
pub enum GamepadButton {
    // actual buttons
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    X,
    Y,
    Plus,
    Minus,
    L,
    R,
    ZL,
    ZR,
    StickL,
    StickR,
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Enum)]
pub enum VirtualGamepadButton {
    StickLUp,
    StickLDown,
    StickLLeft,
    StickLRight,
    StickRUp,
    StickRDown,
    StickRLeft,
    StickRRight,
}

impl GamepadButton {
    pub fn from_gilrs(button: gilrs::Button) -> Option<Self> {
        use gilrs::Button::*;
        Some(match button {
            South => GamepadButton::B,
            East => GamepadButton::A,
            West => GamepadButton::Y,
            North => GamepadButton::X,
            Start => GamepadButton::Plus,
            Select => GamepadButton::Minus,
            LeftTrigger => GamepadButton::L,
            RightTrigger => GamepadButton::R,
            LeftTrigger2 => GamepadButton::ZL,
            RightTrigger2 => GamepadButton::ZR,
            LeftThumb => GamepadButton::StickL,
            RightThumb => GamepadButton::StickR,
            DPadUp => GamepadButton::Up,
            DPadDown => GamepadButton::Down,
            DPadLeft => GamepadButton::Left,
            DPadRight => GamepadButton::Right,
            _ => return None,
        })
    }
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Enum)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Right mouse button
    Right,
    /// Middle mouse button (pressing the scroll wheel)
    Middle,
    /// Wheel up pseudo-button (scrolling up, discrete)
    WheelUp,
    /// Wheel down pseudo-button (scrolling down, discrete)
    WheelDown,
    // Ignore "other" mouse buttons for the sake of simplicity
    // Other(u16),
}
