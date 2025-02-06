// this is noisy & not well-supported by IDEs
#![allow(clippy::uninlined_format_args)]

extern crate self as shin;

use clap::Parser;
use tracing::error;

#[expect(unused)]
mod adv;
mod app;
mod asset;
mod audio;
mod cli;
#[expect(unused)]
mod fps_counter;
mod layer;
mod render;
mod time;
mod update;
mod wiper;
// mod window;

fn main() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_error_panic_hook::set_once();
            tracing_wasm::set_as_global_default();
        } else {
            tracing_subscriber::fmt::init();
        }
    }
    let cli = cli::Cli::parse();

    // Create a background thread which checks for deadlocks every 10s
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
        let deadlocks = parking_lot::deadlock::check_deadlock();
        if deadlocks.is_empty() {
            continue;
        }

        error!("{} deadlocks detected", deadlocks.len());
        for (i, threads) in deadlocks.iter().enumerate() {
            error!("Deadlock #{}", i);
            for t in threads {
                error!("Thread Id {:#?}", t.thread_id());
                error!("{:#?}", t.backtrace());
            }
        }
    });

    shin_tasks::create_task_pools();

    shin_window::run_window::<app::App>(cli);
}
