use std::time::Duration;

use dpi::PhysicalSize;
use enum_map::{enum_map, Enum, EnumMap};
use glam::{Mat4, Vec3};
use shin_input::{inputs::GamepadButton, Action, ActionState, RawInputState};
use shin_render::{
    render_pass::RenderPass,
    resize::ViewportParams,
    shaders::types::{
        buffer::VertexSource,
        vertices::{FloatColor4, PosColVertex, PosVertex, UnormColor},
    },
    DrawPrimitive, RenderProgramWithArguments, RenderRequestBuilder,
};
use shin_window::{AppContext, ShinApp};
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

    fn init(_context: AppContext<Self>, (): Self::Parameters) -> Self {
        HelloApp {}
    }

    fn map_canvas_size(window_size: PhysicalSize<u32>) -> ViewportParams {
        ViewportParams::with_aspect_ratio(window_size, 16.0 / 9.0)
    }

    fn custom_event(&mut self, _context: AppContext<Self>, (): Self::EventType) {}

    fn update(
        &mut self,
        context: AppContext<Self>,
        input: EnumMap<HelloAction, ActionState>,
        _elapsed: Duration,
    ) {
        if input[HelloAction::Back].is_clicked {
            context.event_loop.exit();
        }
    }

    fn render(&mut self, pass: &mut RenderPass) {
        let vertices = [
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
                vertices: VertexSource::VertexData {
                    vertex_data: &vertices,
                },
                color: FloatColor4::BLUE,
            },
            DrawPrimitive::TrianglesStrip,
        ));

        let vertices = [
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
                vertices: VertexSource::VertexData {
                    vertex_data: &vertices,
                },
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
