use std::time::{Duration, Instant};

use anyhow::Result;
use crossbeam_channel::Sender;
use tokio::sync::watch;
use web_engine_api::{EngineCommand, RenderFrame};

pub fn run(
    mut frames: watch::Receiver<Option<RenderFrame>>,
    commands: &Sender<EngineCommand>,
) -> Result<()> {
    let frame_interval = Duration::from_nanos(1_000_000_000 / 120);
    let started = Instant::now();
    let mut next_frame = started;
    let mut last_sequence = 0;

    // This bounded, non-blocking loop stands in for a platform UI event loop.
    // Engine work and IPC are on other threads; UI work never waits for a page.
    while started.elapsed() < Duration::from_secs(1) {
        if let Some(frame) = frames.borrow_and_update().clone() {
            last_sequence = frame.sequence;
            // Framebuffer handoff belongs at the renderer boundary: import the
            // shared OS/GPU resource when supported, otherwise upload CPU bytes.
            // GPU resources are device/context-specific; never assume a texture
            // from the engine can be used directly by the shell's `wgpu::Device`.
            let _frame_dimensions = (frame.width, frame.height);
            let _payload = frame.payload;
        }

        let now = Instant::now();
        if next_frame > now {
            std::thread::sleep(next_frame - now);
        }
        next_frame += frame_interval;
    }

    eprintln!("UI loop presented frames through sequence {last_sequence}");
    commands
        .send(EngineCommand::Shutdown)
        .map_err(|error| anyhow::anyhow!("sending engine shutdown: {error}"))?;
    Ok(())
}
