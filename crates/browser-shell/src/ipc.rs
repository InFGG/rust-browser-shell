use anyhow::{Context, Result};
use crossbeam_channel::Receiver;
use tokio::sync::{mpsc, watch};
use web_engine_api::RenderFrame;

pub fn run(
    engine_frames: Receiver<RenderFrame>,
    ui_frames: watch::Sender<Option<RenderFrame>>,
) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .context("building IPC Tokio runtime")?;

    runtime.block_on(async move {
        let (ipc_tx, mut ipc_rx) = mpsc::channel::<RenderFrame>(2);

        // The channel here represents the transport boundary. Replace this bridge
        // with framed IPC/shared-memory handles when the engine runs out of process.
        let bridge = std::thread::Builder::new()
            .name("engine-frame-bridge".into())
            .spawn(move || {
                while let Ok(frame) = engine_frames.recv() {
                    if ipc_tx.blocking_send(frame).is_err() {
                        break;
                    }
                }
            })
            .context("spawning engine frame bridge")?;

        'forward: while let Some(frame) = ipc_rx.recv().await {
            // Watch retains only the newest frame, so slow presentation cannot
            // build latency or an unbounded queue of stale frames.
            if ui_frames.receiver_count() == 0 {
                break 'forward;
            }
            ui_frames.send_replace(Some(frame));
        }

        bridge
            .join()
            .map_err(|_| anyhow::anyhow!("engine frame bridge panicked"))?;
        Ok(())
    })
}
