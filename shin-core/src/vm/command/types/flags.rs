use bitflags::bitflags;
use proc_bitfield::bitfield;

use crate::format::scenario::instruction_elements::FromNumber;

bitfield! {
    /// Flags that can be used in [LAYERCTRL](super::super::runtime::LAYERCTRL) command
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct LayerCtrlFlags(pub i32) : Debug {
        /// Which easing function to use (see [Easing](crate::time::Easing))
        pub easing: i32 @ 0..6,
        pub scale_time: bool @ 6,
        /// If true - the target value is relative to the current value
        pub delta: bool @ 7,
        pub ff_to_current: bool @ 8,
        pub ff_to_target: bool @ 9,
        pub unused_1: i32 @ 10..12,
        pub prohibit_fast_forward: bool @ 12,
        pub unused_2: i32 @ 13..16,
        pub ignore_wait: bool @ 16,
        pub unused_3: i32 @ 17..32,
    }
}

impl FromNumber for LayerCtrlFlags {
    fn from_number(number: i32) -> Self {
        Self(number)
    }
}

bitflags! {
    /// Flags that can be used in [MASKLOAD](super::super::runtime::MASKLOAD) command and with `MaskWiper`
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct MaskFlags: i32 {
        const FLIP_X = 0x0001;
        const FLIP_Y = 0x0002;
        const FLIP_MIN_MAX = 0x0004;
        const SCALE = 0x0010;
    }
}

impl FromNumber for MaskFlags {
    fn from_number(number: i32) -> Self {
        MaskFlags::from_bits(number).expect("Invalid MaskFlags")
    }
}

bitflags! {
    /// Represents a status of a playing audio that can be awaited on
    ///
    /// Used in [BGMWAIT](super::super::runtime::BGMWAIT), [SEWAIT](super::super::runtime::SEWAIT) and [VOICEWAIT](super::super::runtime::VOICEWAIT) commands
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct AudioWaitStatus: i32 {
        // Not sure about this name tbh...
        // I _think_ it's set while the sound is still fading in
        const FADING = 1;
        const PLAYING = 2;
        const VOLUME_TWEENING = 4;
        const PANNING_TWEENING = 8;
        const PLAY_SPEED_TWEENING = 16;
    }
}

impl FromNumber for AudioWaitStatus {
    fn from_number(number: i32) -> Self {
        AudioWaitStatus::from_bits(number).expect("Invalid AudioWaitStatus")
    }
}

bitflags! {
    /// Flags modifying LAYERLOAD behavior. Unused in umi
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct LayerLoadFlags: i32 {
        /// Prevents setting some flag in ADV
        const DONT_BLOCK_ANIMATIONS = 1;
        /// Keep previous layer parameters
        const KEEP_PREVIOUS_PROPERTIES = 2;
        /// Individually wipe the layer when adding it to the LayerGroup and wait for wipe completion. Ignored if PAGEBACK is active
        const AUTO_WIPE = 4;
        /// Makes the layer share the `Properties` instance with the previous layer while doing `LayerGroup`-level transition
        // We are definitely not implementing that
        // 1. Umineko doesn't use it, along with the whole `LayerGroup`-level transition system
        // 2. We don't have reference counting on everything, so this will be disruptive
        const SHARE_PROPS_DURING_WIPE = 8;
    }
}

impl FromNumber for LayerLoadFlags {
    fn from_number(number: i32) -> Self {
        LayerLoadFlags::from_bits(number).expect("Invalid LayerLoadFlags")
    }
}

bitflags! {
    /// Flags modifying WIPE behavior
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct WipeFlags: i32 {
        const DONT_BLOCK_ANIMATIONS = 1;
        const DONT_WAIT = 2;
    }
}

impl FromNumber for WipeFlags {
    fn from_number(number: i32) -> Self {
        WipeFlags::from_bits(number).expect("Invalid WipeFlags")
    }
}
