use std::time::Duration;

use enum_map::{Enum, EnumMap, enum_map};
use glam::{Mat4, Vec3};
use shin_input::{Action, ActionState, RawInputState, inputs::GamepadButton};
use shin_primitives::color::{FloatColor4, UnormColor};
use shin_render::{
    DrawPrimitive, RenderProgramWithArguments, RenderRequestBuilder,
    render_pass::RenderPass,
    shaders::types::{
        buffer::VertexSource,
        vertices::{PosColVertex, PosVertex},
    },
};
use shin_window::{AppContext, RenderContext, ShinApp};
use winit::keyboard::KeyCode;

#[derive(Enum)]
enum HelloAction {
    Ok,
    Back,
}

impl Action for HelloAction {
    fn lower(
        RawInputState {
            mouse: _,
            keyboard,
            gamepads,
        }: &RawInputState,
    ) -> EnumMap<Self, bool> {
        enum_map! {
            HelloAction::Ok => keyboard.contains(&KeyCode::Enter) || keyboard.contains(&KeyCode::Space) || gamepads.is_held(GamepadButton::A),
            HelloAction::Back => keyboard.contains(&KeyCode::KeyQ) || keyboard.contains(&KeyCode::Escape) || gamepads.is_held(GamepadButton::B),
        }
    }
}

struct HelloApp {}

impl ShinApp for HelloApp {
    type Parameters = ();
    type EventType = ();
    type ActionType = HelloAction;

    fn init(_context: AppContext<Self>, (): Self::Parameters) -> anyhow::Result<Self> {
        Ok(HelloApp {})
    }

    fn custom_event(&mut self, _context: AppContext<Self>, (): Self::EventType) {}

    fn update(
        &mut self,
        context: AppContext<Self>,
        input: EnumMap<HelloAction, ActionState>,
        _elapsed: Duration,
        _command_encoder: &mut wgpu::CommandEncoder,
    ) {
        if input[HelloAction::Back].is_clicked {
            context.event_loop.exit();
        }
    }

    fn render(&mut self, _context: RenderContext, pass: &mut RenderPass) {
        let vertices = &[
            PosVertex {
                position: Vec3::new(-1.0, -1.0, 0.0),
            },
            PosVertex {
                position: Vec3::new(1.0, -1.0, 0.0),
            },
            PosVertex {
                position: Vec3::new(-1.0, 1.0, 0.0),
            },
            PosVertex {
                position: Vec3::new(1.0, 1.0, 0.0),
            },
        ];

        pass.run(RenderRequestBuilder::new().build(
            RenderProgramWithArguments::Clear {
                vertices: VertexSource::VertexData { vertices },
                color: FloatColor4::BLUE,
            },
            DrawPrimitive::TrianglesStrip,
        ));

        let vertices = &[
            PosColVertex {
                position: Vec3::new(0.0, 0.5, 0.0),
                color: UnormColor::RED,
            },
            PosColVertex {
                position: Vec3::new(-0.5, -0.5, 0.0),
                color: UnormColor::GREEN,
            },
            PosColVertex {
                position: Vec3::new(0.5, -0.5, 0.0),
                color: UnormColor::BLUE,
            },
        ];

        pass.run(RenderRequestBuilder::new().build(
            RenderProgramWithArguments::Fill {
                vertices: VertexSource::VertexData { vertices },
                transform: Mat4::IDENTITY,
            },
            DrawPrimitive::Triangles,
        ));
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    #[cfg(target_arch = "wasm32")]
    tracing_wasm::set_as_global_default();

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    shin_tasks::create_task_pools();

    shin_window::run_window::<HelloApp>(());
}
