#![expect(clippy::derivable_impls)] // want to be explicit

//! Rendering framework for the shin engine.

// here we create an abstraction over wgpu which makes it look more like shin's render abstraction over nvn.
// an important departure is not using global variables, but making all the arguments explicit (helped by a builder pattern with typestates (maybe))

mod depth_stencil;
pub mod dynamic_buffer;
pub mod gpu_texture;
pub mod init;
pub mod pipelines;
pub mod quad_vertices;
pub mod render_pass;
pub mod render_texture;
pub mod resize;
pub mod resizeable_texture;

use enum_iterator::Sequence;
use glam::{Mat4, Vec2, Vec3, Vec4, vec3, vec4};
use shin_core::primitives::color::FloatColor4;
use shin_render_shader_types::{
    buffer::VertexSource,
    texture::TextureSource,
    vertices::{MaskVertex, PosColTexVertex, PosColVertex, PosTexVertex, PosVertex, TextVertex},
};
pub use shin_render_shaders as shaders;
use shin_render_shaders::ShaderName;

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_STENCIL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PassKind {
    Opaque,
    Transparent,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
#[repr(u32)]
pub enum LayerShaderOutputKind {
    Layer,
    LayerPremultiply,
    LayerDiscard,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
#[repr(u32)]
pub enum LayerFragmentShader {
    Default = 0,
    Mono = 1,
    Fill = 2,
    Fill2 = 3,
    Negative = 4,
    Gamma = 5,
}

impl LayerFragmentShader {
    pub fn is_equivalent_to_default(self, param: Vec4) -> bool {
        match self {
            LayerFragmentShader::Default => true,
            LayerFragmentShader::Mono => param == vec4(1.0, 1.0, 1.0, 0.0),
            LayerFragmentShader::Fill => param.w == 0.0,
            LayerFragmentShader::Fill2 => param.truncate() == Vec3::ZERO,
            LayerFragmentShader::Negative => false,
            LayerFragmentShader::Gamma => param.truncate() == Vec3::ONE,
        }
    }

    /// If the shader is equivalent to the default shader with the given parameters, downgrades to the default shader.
    pub fn simplify(self, param: Vec4) -> Self {
        if self.is_equivalent_to_default(param) {
            LayerFragmentShader::Default
        } else {
            self
        }
    }

    pub fn evaluate(self, color: FloatColor4, param: Vec4) -> FloatColor4 {
        let color = color.into_vec4();

        let color = match self {
            LayerFragmentShader::Default => color,
            LayerFragmentShader::Mono => {
                let luma = color.truncate().dot(vec3(0.299, 0.587, 0.114));

                Vec3::splat(luma).extend(color.w) * param.truncate().extend(1.0)
            }
            LayerFragmentShader::Fill => {
                // Vec4::lerp()
                todo!()
            }
            LayerFragmentShader::Fill2 => {
                todo!()
            }
            LayerFragmentShader::Negative => {
                todo!()
            }
            LayerFragmentShader::Gamma => {
                todo!()
            }
        };

        FloatColor4::from_vec4(color)
    }
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
        sprite: TextureSource<'a>,
        transform: Mat4,
    },
    Font {
        vertices: VertexSource<'a, TextVertex>,
        glyph: TextureSource<'a>,
        transform: Mat4,
        color1: FloatColor4,
        color2: FloatColor4,
    },
    FontBorder {
        vertices: VertexSource<'a, TextVertex>,
        glyph: TextureSource<'a>,
        transform: Mat4,
        distances: [Vec2; 8],
        color: FloatColor4,
    },
    Button {
        vertices: VertexSource<'a, PosColTexVertex>,
        texture: TextureSource<'a>,
        transform: Mat4,
        // IDK what is the difference between those, seems like they do the same thing repeatedly?
        // higurashi and dc4 only have one of those
        flash1: Vec4,
        flash2: Vec4,
    },
    Blend {
        // TODO
        // vertices: VertexSource<'a, NewBlendVertex>,
        texture1: TextureSource<'a>,
        texture2: TextureSource<'a>,
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
        fragment_shader: LayerFragmentShader,
        vertices: VertexSource<'a, PosTexVertex>,
        texture: TextureSource<'a>,
        transform: Mat4,
        color_multiplier: FloatColor4,
        fragment_shader_param: Vec4,
    },
    Mask {
        fragment_shader: LayerFragmentShader,
        vertices: VertexSource<'a, MaskVertex>,
        texture: TextureSource<'a>,
        mask: TextureSource<'a>,
        transform: Mat4,
        color_multiplier: FloatColor4,
        fragment_shader_param: Vec4,
        minmax: Vec2,
    },

    Dissolve {},
    TapEffect {},
    Movie {
        vertices: VertexSource<'a, PosTexVertex>,
        texture_luma: TextureSource<'a>,
        texture_chroma: TextureSource<'a>,
        transform: Mat4,
        color_bias: Vec4,
        color_transform: [Vec4; 3],
    },
    MovieAlpha {},

    WiperDefault {
        vertices: VertexSource<'a, PosTexVertex>,
        texture_source: TextureSource<'a>,
        texture_target: TextureSource<'a>,
        transform: Mat4,
        alpha: f32,
    },
    WiperMask {
        vertices: VertexSource<'a, MaskVertex>,
        texture_source: TextureSource<'a>,
        texture_target: TextureSource<'a>,
        texture_mask: TextureSource<'a>,
        transform: Mat4,
        minmax: Vec2,
    },
    WiperWave {
        // TODO
    },
    WiperRipple {
        // TODO
    },
    WiperWhirl {
        // TODO
    },
    WiperGlass {
        // TODO
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
    pub function: DepthFunction,
    pub write_enable: bool,
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
    SignOnly = 0x80,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Sequence)]
pub struct StencilPipelineState {
    pub function: StencilFunction,
    pub stencil_fail_operation: StencilOperation,
    pub depth_fail_operation: StencilOperation,
    pub pass_operation: StencilOperation,
    pub stencil_read_mask: StencilMask,
    pub stencil_write_mask: StencilMask,
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
    pub pipeline: StencilPipelineState,
    pub stencil_reference: u8,
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
    pub depth: DepthState,
    pub stencil: StencilPipelineState,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct DepthStencilState {
    pub depth: DepthState,
    pub stencil: StencilState,
}

impl DepthStencilState {
    pub fn shorthand(stencil_ref: u8, allow_eq_stencil: bool, test_depth: bool) -> Self {
        let depth = if test_depth {
            DepthState {
                function: DepthFunction::Less,
                write_enable: false,
            }
        } else {
            DepthState::default()
        };
        let stencil = StencilState {
            pipeline: StencilPipelineState {
                function: if allow_eq_stencil {
                    StencilFunction::GreaterOrEqual
                } else {
                    StencilFunction::Greater
                },
                pass_operation: StencilOperation::Replace,
                ..Default::default()
            },
            stencil_reference: stencil_ref,
        };

        Self { depth, stencil }
    }

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

impl ColorBlendType {
    pub fn from_regular_layer(layer: LayerBlendType) -> Self {
        match layer {
            LayerBlendType::Type1 => ColorBlendType::Layer1,
            LayerBlendType::Type2 => ColorBlendType::Layer2,
            LayerBlendType::Type3 => ColorBlendType::Layer3,
        }
    }

    pub fn from_premultiplied_layer(layer: LayerBlendType) -> Self {
        match layer {
            LayerBlendType::Type1 => ColorBlendType::LayerPremultiplied1,
            LayerBlendType::Type2 => ColorBlendType::LayerPremultiplied2,
            LayerBlendType::Type3 => ColorBlendType::LayerPremultiplied3,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum LayerBlendType {
    Type1,
    Type2,
    Type3,
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

#[derive(Default, Copy, Clone)]
#[must_use]
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

    pub fn depth_stencil_shorthand(
        mut self,
        stencil_ref: u8,
        allow_eq_stencil: bool,
        test_depth: bool,
    ) -> Self {
        self.depth_stencil =
            DepthStencilState::shorthand(stencil_ref, allow_eq_stencil, test_depth);
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

    pub fn layer_color_blend(mut self, color_blend_type: LayerBlendType) -> Self {
        self.color_blend_type = ColorBlendType::from_regular_layer(color_blend_type);
        self
    }

    pub fn layer_color_blend_premultiplied(mut self, color_blend_type: LayerBlendType) -> Self {
        self.color_blend_type = ColorBlendType::from_premultiplied_layer(color_blend_type);
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

#[must_use]
pub struct RenderRequest<'a> {
    depth_stencil: DepthStencilState,
    color_blend_type: ColorBlendType,
    cull_faces: CullFace,
    primitive: DrawPrimitive,
    program: RenderProgramWithArguments<'a>,
}

pub fn shin_orthographic_projection_matrix(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    // flip the y coordinate (because the game uses coordinate system with Y going down, but wgpu has Y going up)
    // and map Z from [-1; 1] to [0; 1] (because glam uses OpenGL's range of [-1; 1], while wgpu has range [0; 1])
    #[rustfmt::skip]
    const SHIN_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0,
        0.0,-1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    ]);

    SHIN_TO_WGPU_MATRIX * Mat4::orthographic_rh_gl(left, right, bottom, top, near, far)
}

// /// A trait for elements that can be rendered
// ///
// /// Most elements will be containers, containing other elements to render.
// pub trait Renderable {
//     /// Renders an element on the screen
//     ///
//     /// # Arguments
//     ///
//     /// * `resources` - The common resources used by the renderer
//     /// * `render_pass` - The render pass to encode commands to
//     /// * `transform` - The transform matrix to apply to the element
//     /// * `projection` - The projection matrix to apply to the element
//     ///
//     /// # Notes
//     ///
//     /// The `projection` matrix is used to convert from virtual screen space to real screen space.
//     /// The `transform` matrix represents inherited transformations from parent elements.
//     ///
//     /// This distinction is needed to allow for rendering using intermediate render targets.
//     /// In this case the `transform` matrix is preserved and passed into inner elements.
//     /// The `projection` matrix is used only to render the intermediate render target to the screen.
//     fn render<'enc>(
//         &'enc self,
//         resources: &'enc GpuCommonResources,
//         render_pass: &mut wgpu::RenderPass<'enc>,
//         transform: Mat4,
//         projection: Mat4,
//     );
//
//     /// Notifies of window resize
//     ///
//     /// If a renderable element has an intermediate render target, it should resize it here.
//     fn resize(&mut self, resources: &GpuCommonResources);
// }
