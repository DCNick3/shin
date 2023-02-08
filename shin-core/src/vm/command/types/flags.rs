use crate::format::scenario::instructions::NumberSpec;
use crate::vm::{FromVmCtx, VmCtx};
use bitflags::bitflags;
use proc_bitfield::bitfield;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct LayerCtrlFlags(pub i32) : Debug {
        pub easing: i32 @ 0..6,
        pub scale_time: bool @ 6,
        pub delta: bool @ 7,
        pub ff_to_current: bool @ 8,
        pub ff_to_target: bool @ 9,
        pub unused_1: i32 @ 10..12,
        pub prohibit_fast_forwward: bool @ 12,
        pub unused_2: i32 @ 13..16,
        pub ignore_wait: bool @ 16,
        pub unused_3: i32 @ 17..32,
    }
}

impl FromVmCtx<NumberSpec> for LayerCtrlFlags {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec) -> Self {
        Self(ctx.get_number(input))
    }
}

bitflags! {
    pub struct MaskFlags: i32 {
        const FLIP_X = 0x0001;
        const FLIP_Y = 0x0002;
        const SCALE = 0x0010;
    }
}

impl FromVmCtx<NumberSpec> for MaskFlags {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec) -> Self {
        MaskFlags::from_bits(ctx.get_number(input)).expect("Invalid MaskFlags")
    }
}

bitflags! {
    pub struct AudioWaitStatus: i32 {
        const PLAYING = 1;
        const STOPPED = 2;
        const VOLUME_TWEENER_IDLE = 4;
        const PANNING_TWEENER_IDLE = 8;
        const PLAY_SPEED_TWEENER_IDLE = 16;
    }
}

impl FromVmCtx<NumberSpec> for AudioWaitStatus {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec) -> Self {
        AudioWaitStatus::from_bits(ctx.get_number(input)).expect("Invalid AudioWaitStatus")
    }
}
