use std::{fs::File, time::Duration};

use enum_map::{enum_map, Enum, EnumMap};
use shin_audio::AudioManager;
use shin_core::{primitives::update::FrameId, time::Ticks};
use shin_input::{
    inputs::{GamepadButton, KeyCode},
    Action, ActionState, RawInputState,
};
use shin_render::render_pass::RenderPass;
use shin_video::{mp4::Mp4, VideoPlayerHandle};
use shin_window::{AppContext, RenderContext, ShinApp};

#[derive(Enum)]
enum PlayAction {
    Exit,
    ToggleFullscreen,
}

impl Action for PlayAction {
    fn lower(
        RawInputState {
            mouse: _,
            keyboard,
            gamepads,
        }: &RawInputState,
    ) -> EnumMap<Self, bool> {
        enum_map! {
            PlayAction::Exit => keyboard.contains(&KeyCode::KeyQ) || keyboard.contains(&KeyCode::Escape) || gamepads.is_held(GamepadButton::Plus),
            PlayAction::ToggleFullscreen => keyboard.contains(&KeyCode::F11),
        }
    }
}

struct PlayerExample {
    #[allow(dead_code)] // it's doing its thing in the background
    audio_manager: AudioManager,
    video_player: VideoPlayerHandle,
    frame: FrameId,
}

impl ShinApp for PlayerExample {
    type Parameters = ();
    type EventType = ();
    type ActionType = PlayAction;

    fn init(context: AppContext<Self>, _parameters: Self::Parameters) -> anyhow::Result<Self> {
        let audio_manager = AudioManager::new();

        // let file = File::open("ship1.mp4").unwrap();
        let file = File::open("op1.mp4").unwrap();
        let mp4 = Mp4::new(file).unwrap();
        let video_player =
            VideoPlayerHandle::new(&context.wgpu.device, &audio_manager, mp4).unwrap();

        Ok(Self {
            audio_manager,
            video_player,
            frame: FrameId::default(),
        })
    }

    fn custom_event(&mut self, _context: AppContext<Self>, _event: Self::EventType) {}

    fn update(
        &mut self,
        context: AppContext<Self>,
        input: EnumMap<Self::ActionType, ActionState>,
        elapsed_time: Duration,
    ) {
        if input[PlayAction::Exit].is_clicked || self.video_player.is_finished() {
            context.event_loop.exit();
        }
        if input[PlayAction::ToggleFullscreen].is_clicked {
            context.winit.toggle_fullscreen();
        }

        self.video_player.update(
            self.frame,
            Ticks::from_duration(elapsed_time),
            &context.wgpu.queue,
        );

        self.frame.advance();
    }

    fn render(&mut self, _context: RenderContext, pass: &mut RenderPass) {
        if let Some(frame) = self.video_player.get_frame() {
            frame.render(pass);
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    shin_tasks::create_task_pools();

    shin_window::run_window::<PlayerExample>(());
}
