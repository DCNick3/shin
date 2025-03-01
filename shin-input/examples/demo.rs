use std::{collections::VecDeque, time::Duration};

use enum_map::{Enum, EnumMap, enum_map};
use glam::{Mat4, Vec3};
use shin_input::{
    Action, ActionState, RawInputState,
    inputs::{GamepadButton, VirtualGamepadButton},
};
use shin_primitives::color::UnormColor;
use shin_render::{
    DrawPrimitive, RenderProgramWithArguments, RenderRequestBuilder,
    render_pass::RenderPass,
    shaders::types::{buffer::VertexSource, vertices::PosColVertex},
};
use shin_window::{AppContext, RenderContext, ShinApp};
use tracing::info;
use winit::keyboard::KeyCode;

#[derive(Enum)]
enum HelloAction {
    Ok,
    Back,
    Up,
    Down,
    Left,
    Right,
    ToggleFullscreen,
    SwitchScene,
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
            HelloAction::Up => keyboard.contains(&KeyCode::ArrowUp) || gamepads.is_held(GamepadButton::Up) || gamepads.is_vheld(VirtualGamepadButton::StickLUp) || gamepads.is_vheld(VirtualGamepadButton::StickRUp),
            HelloAction::Down => keyboard.contains(&KeyCode::ArrowDown) || gamepads.is_held(GamepadButton::Down) || gamepads.is_vheld(VirtualGamepadButton::StickLDown) || gamepads.is_vheld(VirtualGamepadButton::StickRDown),
            HelloAction::Left => keyboard.contains(&KeyCode::ArrowLeft) || gamepads.is_held(GamepadButton::Left) || gamepads.is_vheld(VirtualGamepadButton::StickLLeft) || gamepads.is_vheld(VirtualGamepadButton::StickRLeft),
            HelloAction::Right => keyboard.contains(&KeyCode::ArrowRight) || gamepads.is_held(GamepadButton::Right) || gamepads.is_vheld(VirtualGamepadButton::StickLRight) || gamepads.is_vheld(VirtualGamepadButton::StickRRight),
            HelloAction::ToggleFullscreen => keyboard.contains(&KeyCode::F11),
            HelloAction::SwitchScene => keyboard.contains(&KeyCode::Tab) || gamepads.is_held(GamepadButton::X),
        }
    }
}

#[derive(Default)]
struct HelloAppInputHistory {
    input_history: VecDeque<EnumMap<HelloAction, ActionState>>,
}

impl HelloAppInputHistory {
    const HISTORY_SIZE: usize = 128;

    fn input_state_to_col(state: EnumMap<HelloAction, ActionState>) -> [bool; 18] {
        fn state_to_group(group: &mut [bool], state: ActionState) {
            group[0] = state.is_held;
            group[1] = state.is_clicked;
            group[2] = state.is_clicked_or_repeated;
            group[3] = state.is_clicked_or_rapid_repeated;
        }

        let mut row = [false; 18];
        state_to_group(&mut row[1..5], state[HelloAction::Up]);
        state_to_group(&mut row[5..9], state[HelloAction::Down]);
        state_to_group(&mut row[9..13], state[HelloAction::Left]);
        state_to_group(&mut row[13..17], state[HelloAction::Right]);

        row
    }

    fn draw_grid(pass: &mut RenderPass, grid: [[bool; 18]; Self::HISTORY_SIZE]) {
        const ACTIVE_COLOR: UnormColor = UnormColor::PASTEL_GREEN;
        const INACTIVE_COLOR: UnormColor = UnormColor::BLACK;

        // no tesselation, just naively generating 6 vertices per cell
        let mut vertices = Vec::new();
        for (x, column) in grid.iter().enumerate() {
            for (y, &cell) in column.iter().enumerate() {
                let color = if cell { ACTIVE_COLOR } else { INACTIVE_COLOR };
                let x = x as f32;
                let y = y as f32;
                vertices.push(PosColVertex {
                    position: Vec3::new(x, y, 0.0),
                    color,
                });
                vertices.push(PosColVertex {
                    position: Vec3::new(x + 1.0, y, 0.0),
                    color,
                });
                vertices.push(PosColVertex {
                    position: Vec3::new(x, y + 1.0, 0.0),
                    color,
                });
                vertices.push(PosColVertex {
                    position: Vec3::new(x + 1.0, y, 0.0),
                    color,
                });
                vertices.push(PosColVertex {
                    position: Vec3::new(x + 1.0, y + 1.0, 0.0),
                    color,
                });
                vertices.push(PosColVertex {
                    position: Vec3::new(x, y + 1.0, 0.0),
                    color,
                });
            }
        }

        pass.run(RenderRequestBuilder::new().build(
            RenderProgramWithArguments::Fill {
                vertices: VertexSource::VertexData {
                    vertices: &vertices,
                },
                // transform to fit the grid into normalized device coordinates
                transform: Mat4::from_translation(Vec3::new(-1.0, -1.0, 0.0))
                    * Mat4::from_scale(Vec3::new(2.0 / Self::HISTORY_SIZE as f32, 2.0 / 18.0, 1.0)),
            },
            DrawPrimitive::Triangles,
        ));
    }

    pub fn update(&mut self, input: EnumMap<HelloAction, ActionState>, _elapsed_time: Duration) {
        self.input_history.push_front(input);
        while self.input_history.len() >= Self::HISTORY_SIZE {
            self.input_history.pop_back();
        }
    }
    pub fn render(&mut self, pass: &mut RenderPass) {
        let grid = self
            .input_history
            .iter()
            .cloned()
            .map(Self::input_state_to_col)
            .chain(std::iter::repeat([false; 18]))
            .take(Self::HISTORY_SIZE)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Self::draw_grid(pass, grid);
    }
}

struct HelloAppMove {
    // position of the centre in virtual 1920x1080 space (0,0) is at the centre of the screen
    block_position: (f32, f32),
}

impl Default for HelloAppMove {
    fn default() -> Self {
        HelloAppMove {
            block_position: (0.0, 0.0),
        }
    }
}

impl HelloAppMove {
    pub fn update(&mut self, input: EnumMap<HelloAction, ActionState>, _elapsed_time: Duration) {
        let block_step_size = 40.0;

        if input[HelloAction::Up].is_clicked_or_rapid_repeated {
            self.block_position.1 -= block_step_size;
        }
        if input[HelloAction::Down].is_clicked_or_rapid_repeated {
            self.block_position.1 += block_step_size;
        }
        if input[HelloAction::Left].is_clicked_or_rapid_repeated {
            self.block_position.0 -= block_step_size;
        }
        if input[HelloAction::Right].is_clicked_or_rapid_repeated {
            self.block_position.0 += block_step_size;
        }

        if self.block_position.0 < -960.0 {
            self.block_position.0 += 1920.0;
        }
        if self.block_position.0 >= 960.0 {
            self.block_position.0 -= 1920.0;
        }
        if self.block_position.1 < -540.0 {
            self.block_position.1 += 1080.0;
        }
        if self.block_position.1 >= 540.0 {
            self.block_position.1 -= 1080.0;
        }
    }
    pub fn render(&mut self, pass: &mut RenderPass) {
        const BLOCK_SIZE: f32 = 20.0;

        let vertices = [
            PosColVertex {
                position: Vec3::new(-10.0, -1.0, 0.0),
                color: UnormColor::PASTEL_PINK,
            },
            PosColVertex {
                position: Vec3::new(10.0, -1.0, 0.0),
                color: UnormColor::PASTEL_PINK,
            },
            PosColVertex {
                position: Vec3::new(-10.0, 1.0, 0.0),
                color: UnormColor::PASTEL_PINK,
            },
            PosColVertex {
                position: Vec3::new(10.0, 1.0, 0.0),
                color: UnormColor::PASTEL_PINK,
            },
        ];

        pass.run(RenderRequestBuilder::new().build(
            RenderProgramWithArguments::Fill {
                vertices: VertexSource::VertexData {
                    vertices: &vertices,
                },
                transform: Mat4::from_scale(Vec3::new(2.0 / 1920.0, 2.0 / 1080.0, 1.0))
                    * Mat4::from_translation(Vec3::new(
                        self.block_position.0,
                        self.block_position.1,
                        0.0,
                    ))
                    * Mat4::from_scale(Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 1.0)),
            },
            DrawPrimitive::TrianglesStrip,
        ));
    }
}

enum Scene {
    History(HelloAppInputHistory),
    Move(HelloAppMove),
}

impl Scene {
    pub fn next(&self) -> Self {
        match self {
            Scene::History(_) => Scene::Move(HelloAppMove::default()),
            Scene::Move(_) => Scene::History(HelloAppInputHistory::default()),
        }
    }
}

struct HelloApp {
    scene: Scene,
    fps: spin_sleep_util::RateReporter,
}

impl ShinApp for HelloApp {
    type Parameters = ();
    type EventType = ();
    type ActionType = HelloAction;

    fn init(_context: AppContext<Self>, (): Self::Parameters) -> anyhow::Result<Self> {
        Ok(HelloApp {
            scene: Scene::History(HelloAppInputHistory {
                input_history: VecDeque::new(),
            }),
            fps: spin_sleep_util::RateReporter::new(std::time::Duration::from_secs(1)),
        })
    }

    fn custom_event(&mut self, _context: AppContext<Self>, (): Self::EventType) {}

    fn update(
        &mut self,
        context: AppContext<HelloApp>,
        input: EnumMap<HelloAction, ActionState>,
        elapsed_time: Duration,
        _command_encoder: &mut wgpu::CommandEncoder,
    ) {
        if input[HelloAction::Back].is_clicked {
            context.event_loop.exit();
        }

        if input[HelloAction::ToggleFullscreen].is_clicked {
            context.winit.toggle_fullscreen();
        }

        if input[HelloAction::SwitchScene].is_clicked {
            self.scene = self.scene.next();
        }

        match &mut self.scene {
            Scene::History(history) => history.update(input, elapsed_time),
            Scene::Move(move_) => move_.update(input, elapsed_time),
        }
    }

    fn render(&mut self, _context: RenderContext, pass: &mut RenderPass) {
        if let Some(fps) = self.fps.increment_and_report() {
            info!("FPS: {}", fps);
        }

        match &mut self.scene {
            Scene::History(history) => history.render(pass),
            Scene::Move(move_) => move_.render(pass),
        }
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
