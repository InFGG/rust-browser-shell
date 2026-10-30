use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use web_engine_api::{EngineCommand, EngineError, FramePayload, RenderFrame, WebEngineAdapter};

pub struct MockEngine;

impl WebEngineAdapter for MockEngine {
    fn run(
        self: Box<Self>,
        commands: Receiver<EngineCommand>,
        frames: Sender<RenderFrame>,
    ) -> Result<(), EngineError> {
        let mut sequence = 0;
        let mut size = (800, 600);
        let mut running = true;

        while running {
            match commands.recv_timeout(Duration::from_millis(8)) {
                Ok(EngineCommand::Navigate(url)) => {
                    eprintln!("mock engine navigating to {url}");
                }
                Ok(EngineCommand::Resize { width, height, .. }) => size = (width, height),
                Ok(EngineCommand::Shutdown) => running = false,
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => running = false,
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
            }

            if !running {
                break;
            }

            sequence += 1;
            let frame = RenderFrame {
                sequence,
                width: size.0,
                height: size.1,
                payload: FramePayload::CpuBytes(Arc::from([])),
            };
            frames
                .send(frame)
                .map_err(|_| EngineError::FrameConsumerDisconnected)?;
        }

        Ok(())
    }
}
