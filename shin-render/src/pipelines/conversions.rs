use crate::{
    ColorBlendType, CullFace, DepthFunction, DrawPrimitive, StencilFunction, StencilMask,
    StencilOperation, StencilPipelineState,
};

impl From<DrawPrimitive> for wgpu::PrimitiveTopology {
    fn from(value: DrawPrimitive) -> Self {
        match value {
            DrawPrimitive::Triangles => wgpu::PrimitiveTopology::TriangleList,
            DrawPrimitive::TrianglesStrip => wgpu::PrimitiveTopology::TriangleStrip,
        }
    }
}

impl From<CullFace> for Option<wgpu::Face> {
    fn from(value: CullFace) -> Self {
        match value {
            CullFace::None => None,
            CullFace::Front => Some(wgpu::Face::Front),
            CullFace::Back => Some(wgpu::Face::Back),
        }
    }
}

impl From<DepthFunction> for wgpu::CompareFunction {
    fn from(value: DepthFunction) -> Self {
        match value {
            DepthFunction::Never => wgpu::CompareFunction::Never,
            DepthFunction::Less => wgpu::CompareFunction::Less,
            DepthFunction::Equal => wgpu::CompareFunction::Equal,
            DepthFunction::LessOrEqual => wgpu::CompareFunction::LessEqual,
            DepthFunction::Greater => wgpu::CompareFunction::Greater,
            DepthFunction::NotEqual => wgpu::CompareFunction::NotEqual,
            DepthFunction::GreaterOrEqual => wgpu::CompareFunction::GreaterEqual,
            DepthFunction::Always => wgpu::CompareFunction::Always,
        }
    }
}

impl From<StencilFunction> for wgpu::CompareFunction {
    fn from(value: StencilFunction) -> Self {
        match value {
            StencilFunction::Never => wgpu::CompareFunction::Never,
            StencilFunction::Less => wgpu::CompareFunction::Less,
            StencilFunction::Equal => wgpu::CompareFunction::Equal,
            StencilFunction::LessOrEqual => wgpu::CompareFunction::LessEqual,
            StencilFunction::Greater => wgpu::CompareFunction::Greater,
            StencilFunction::NotEqual => wgpu::CompareFunction::NotEqual,
            StencilFunction::GreaterOrEqual => wgpu::CompareFunction::GreaterEqual,
            StencilFunction::Always => wgpu::CompareFunction::Always,
        }
    }
}

impl From<StencilOperation> for wgpu::StencilOperation {
    fn from(value: StencilOperation) -> Self {
        match value {
            StencilOperation::Keep => wgpu::StencilOperation::Keep,
            StencilOperation::Zero => wgpu::StencilOperation::Zero,
            StencilOperation::Replace => wgpu::StencilOperation::Replace,
            StencilOperation::Increment => wgpu::StencilOperation::IncrementClamp,
            StencilOperation::Decrement => wgpu::StencilOperation::DecrementClamp,
            StencilOperation::Invert => wgpu::StencilOperation::Invert,
            StencilOperation::IncrementWrap => wgpu::StencilOperation::IncrementWrap,
            StencilOperation::DecrementWrap => wgpu::StencilOperation::DecrementWrap,
        }
    }
}

impl From<StencilMask> for u32 {
    fn from(value: StencilMask) -> Self {
        match value {
            StencilMask::All => 0xff,
            StencilMask::SignOnly => 0x80,
        }
    }
}

impl From<StencilPipelineState> for wgpu::StencilFaceState {
    fn from(value: StencilPipelineState) -> Self {
        wgpu::StencilFaceState {
            compare: value.function.into(),
            fail_op: value.stencil_fail_operation.into(),
            depth_fail_op: value.depth_fail_operation.into(),
            pass_op: value.pass_operation.into(),
        }
    }
}

impl From<ColorBlendType> for wgpu::BlendState {
    fn from(value: ColorBlendType) -> Self {
        use wgpu::{BlendFactor, BlendOperation, BlendState};

        let layer_alpha = wgpu::BlendComponent {
            src_factor: BlendFactor::OneMinusDstAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        };

        match value {
            ColorBlendType::NoColor | ColorBlendType::Opaque => BlendState {
                color: wgpu::BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::Zero,
                    operation: BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::Zero,
                    operation: BlendOperation::Add,
                },
            },
            ColorBlendType::Layer1 => BlendState {
                color: wgpu::BlendComponent {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
                },
                alpha: layer_alpha,
            },
            ColorBlendType::Layer2 => BlendState {
                color: wgpu::BlendComponent {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                alpha: layer_alpha,
            },
            ColorBlendType::Layer3 => BlendState {
                color: wgpu::BlendComponent {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Subtract,
                },
                alpha: layer_alpha,
            },
            ColorBlendType::LayerPremultiplied1 => BlendState {
                color: wgpu::BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
                },
                alpha: layer_alpha,
            },
            ColorBlendType::LayerPremultiplied2 => BlendState {
                color: wgpu::BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                alpha: layer_alpha,
            },
            ColorBlendType::LayerPremultiplied3 => BlendState {
                color: wgpu::BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::ReverseSubtract,
                },
                alpha: layer_alpha,
            },
        }
    }
}

impl From<ColorBlendType> for wgpu::ColorWrites {
    fn from(value: ColorBlendType) -> Self {
        match value {
            ColorBlendType::NoColor => wgpu::ColorWrites::empty(),
            ColorBlendType::Opaque
            | ColorBlendType::Layer1
            | ColorBlendType::Layer2
            | ColorBlendType::Layer3
            | ColorBlendType::LayerPremultiplied1
            | ColorBlendType::LayerPremultiplied2
            | ColorBlendType::LayerPremultiplied3 => wgpu::ColorWrites::all(),
        }
    }
}
