// here we create an abstraction over wgpu which makes it look more like shin's render abstraction over nvn.
// an important departure is not using global variables, but making all the arguments explicit (helped by a builder pattern with typestates (maybe))

pub mod init;
pub mod pipelines;
pub mod render_pass;
pub mod resize;
pub mod resizeable_texture;

use enum_iterator::Sequence;
use glam::{Mat4, Vec4};
use shin_render_shader_types::{
    buffer::VertexSource,
    texture::TextureBindGroup,
    vertices::{FloatColor4, PosColTexVertex, PosColVertex, PosVertex, TextVertex, VertexType},
};
use shin_render_shaders::ShaderName;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub enum LayerShaderOutputKind {
    Layer,
    LayerDiscard,
    LayerPremultiply,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub enum LayerShaderOperation {
    Default,
    Mono,
    Fill,
    Fill2,
    Negative,
    Gamma,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub enum WiperKind {
    Default,
    Mask,
    Wave,
    Ripple,
    Whirl,
    Glass,
}

// not yet sure if a separate type is needed
pub type TextureSource = TextureBindGroup;

#[derive(Debug)]
pub enum RenderProgramWithArguments<'a> {
    // TODO: specify all arguments required for all programs
    Clear {
        vertices: VertexSource<'a, PosVertex>,
        // implementation note:
        // a lot of the params passed by-value here would go into a Params struct and bound as a uniform buffer for the shaders to use.
        // us accepting these params by-value instead limits the potential performance of the renderer (since with our current design we will have to create a new buffer every time we want to change the color).
        // but allowing to pass direct GPU buffers here would be a detriment for usability and won't lead to better performance, since most of these parameters are computed on the fly either way.
        // this is unlike vertex and index buffers, which can be re-used between frames for some components (and are accepting GPU-side buffers instead).
        color: FloatColor4,
    },
    Fill {
        vertices: VertexSource<'a, PosColVertex>,
        transform: Mat4,
    },
    Sprite {
        vertices: VertexSource<'a, PosColTexVertex>,
        sprite: TextureSource,
        transform: Mat4,
    },
    Font {
        vertices: VertexSource<'a, TextVertex>,
        texture: TextureSource,
        transform: Mat4,
        color1: FloatColor4,
        color2: FloatColor4,
    },
    FontBorder {
        vertices: VertexSource<'a, TextVertex>,
        texture: TextureSource,
        transform: Mat4,
        distances: [Vec4; 4],
        color: FloatColor4,
    },
    Button {
        vertices: VertexSource<'a, PosColTexVertex>,
        texture: TextureSource,
        transform: Mat4,
        // IDK what is the difference between those, seems like they do the same thing repeatedly?
        // higurashi and dc4 only have one of those
        flash1: Vec4,
        flash2: Vec4,
    },
    Blend {
        // TODO
        // vertices: VertexSource<'a, NewBlendVertex>,
        texture1: TextureSource,
        texture2: TextureSource,
        transform: Mat4,
        blend: Vec4,
        flash: Vec4,
    },
    Window {
        // TODO
        // vertices: VertexSource<NewWindowVertex>,
    },

    Layer {
        output_kind: LayerShaderOutputKind,
        operation: LayerShaderOperation,
        // TODO
        // vertices: VertexSource<'a, NewLayerVertex>,
        texture: TextureSource,
        transform: Mat4,
        color: FloatColor4,
        fragment_shader_param: Vec4,
    },
    Mask {
        operation: LayerShaderOperation,
        // TODO
        // vertices: VertexSource<'a, NewMaskVertex>,
        texture1: TextureSource,
        texture2: TextureSource,
        transform: Mat4,
        color: FloatColor4,
        fragment_shader_param: Vec4,
        minmax: Vec4,
    },

    Dissolve {},
    TapEffect {},
    Movie {},
    MovieAlpha {},

    Wiper {
        kind: WiperKind,
    },

    Mosaic {},
    Blur {},
    ZoomBlur {},
    Raster {},
    Ripple {},
    Breakup {},

    Charicon0 {},
    Charicon1 {},
    Charicon2 {},
    Charicon3 {},
    Test {},
}

impl RenderProgramWithArguments<'_> {
    pub fn get_shader_name(&self) -> ShaderName {
        match *self {
            RenderProgramWithArguments::Clear { .. } => ShaderName::Clear,
            RenderProgramWithArguments::Fill { .. } => ShaderName::Fill,
            RenderProgramWithArguments::Sprite { .. } => ShaderName::Sprite,
            RenderProgramWithArguments::Font { .. } => ShaderName::Font,
            ref program => todo!("Implement shader for {:?}", program),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub enum DepthFunction {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    #[default]
    Always,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub struct DepthState {
    function: DepthFunction,
    write_enable: bool,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub enum StencilFunction {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    #[default]
    Always,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub enum StencilOperation {
    #[default]
    Keep,
    Zero,
    Replace,
    Increment,
    Decrement,
    Invert,
    IncrementWrap,
    DecrementWrap,
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
#[repr(u8)]
pub enum StencilMask {
    #[default]
    All = 0xff,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub struct StencilPipelineState {
    function: StencilFunction,
    stencil_fail_operation: StencilOperation,
    depth_fail_operation: StencilOperation,
    pass_operation: StencilOperation,
    stencil_read_mask: StencilMask,
    stencil_write_mask: StencilMask,
}

impl Default for StencilPipelineState {
    fn default() -> Self {
        Self {
            function: StencilFunction::default(),
            stencil_fail_operation: StencilOperation::default(),
            depth_fail_operation: StencilOperation::default(),
            pass_operation: StencilOperation::default(),
            stencil_read_mask: StencilMask::All,
            stencil_write_mask: StencilMask::All,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct StencilState {
    pipeline: StencilPipelineState,
    stencil_reference: u8,
}

impl Default for StencilState {
    fn default() -> Self {
        Self {
            pipeline: StencilPipelineState::default(),
            stencil_reference: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub struct DepthStencilPipelineState {
    depth: DepthState,
    stencil: StencilPipelineState,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct DepthStencilState {
    depth: DepthState,
    stencil: StencilState,
}

impl DepthStencilState {
    pub fn into_pipeline_parts(self) -> (DepthStencilPipelineState, u8) {
        (
            DepthStencilPipelineState {
                depth: self.depth,
                stencil: self.stencil.pipeline,
            },
            self.stencil.stencil_reference,
        )
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub enum ColorBlendType {
    NoColor,
    #[default]
    Opaque,
    // I don't understand the meaning of each of these (yet)
    Layer1,
    Layer2,
    Layer3,
    // those do the same thing as above, but operate on premultiplied alpha (QUESTION: is it on the input or output or both?)
    LayerPremultiplied1,
    LayerPremultiplied2,
    LayerPremultiplied3,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub enum CullFace {
    #[default]
    None,
    Back,
    Front,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub enum DrawPrimitive {
    Triangles,
    TrianglesStrip,
}

#[derive(Default)]
pub struct RenderRequestBuilder {
    depth_stencil: DepthStencilState,
    color_blend_type: ColorBlendType,
    cull_faces: CullFace,
}

impl RenderRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn depth_stencil(mut self, depth_stencil: DepthStencilState) -> Self {
        self.depth_stencil = depth_stencil;
        self
    }

    pub fn cull_faces(mut self, cull_faces: CullFace) -> Self {
        self.cull_faces = cull_faces;
        self
    }

    pub fn color_blend_type(mut self, color_blend_type: ColorBlendType) -> Self {
        self.color_blend_type = color_blend_type;
        self
    }

    pub fn build(
        self,
        program: RenderProgramWithArguments,
        primitive: DrawPrimitive,
    ) -> RenderRequest {
        RenderRequest {
            depth_stencil: self.depth_stencil,
            color_blend_type: self.color_blend_type,
            cull_faces: self.cull_faces,
            primitive,
            program,
        }
    }
}

pub struct RenderRequest<'a> {
    depth_stencil: DepthStencilState,
    color_blend_type: ColorBlendType,
    cull_faces: CullFace,
    primitive: DrawPrimitive,
    program: RenderProgramWithArguments<'a>,
}
