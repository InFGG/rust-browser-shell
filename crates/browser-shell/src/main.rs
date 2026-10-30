mod ipc;
mod mock_engine;
mod ui;

use anyhow::{Context, Result};
use crossbeam_channel::bounded;
use tokio::sync::watch;
use web_engine_api::{EngineCommand, RenderFrame, WebEngineAdapter};

fn main() -> Result<()> {
    let (command_tx, command_rx) = bounded(8);
    let (engine_frame_tx, engine_frame_rx) = bounded::<RenderFrame>(2);
    let (ui_frame_tx, ui_frame_rx) = watch::channel::<Option<RenderFrame>>(None);

    // Keep adapter construction and execution inside its dedicated worker.
    let engine = std::thread::Builder::new()
        .name("web-engine-runner".into())
        .spawn(move || {
            let adapter = Box::new(mock_engine::MockEngine) as Box<dyn WebEngineAdapter>;
            adapter.run(command_rx, engine_frame_tx)
        })
        .context("spawning web engine runner")?;

    let ipc = std::thread::Builder::new()
        .name("async-ipc".into())
        .spawn(move || ipc::run(engine_frame_rx, ui_frame_tx))
        .context("spawning asynchronous IPC loop")?;

    command_tx
        .send(EngineCommand::Navigate("https://example.com".to_owned()))
        .context("sending initial navigation")?;

    // A real native UI framework should own this thread/event loop.
    let ui_result = ui::run(ui_frame_rx, &command_tx);

    engine
        .join()
        .map_err(|_| anyhow::anyhow!("web engine thread panicked"))?
        .context("web engine exited with an error")?;
    ipc.join()
        .map_err(|_| anyhow::anyhow!("IPC thread panicked"))?
        .context("IPC loop exited with an error")?;
    ui_result
}
