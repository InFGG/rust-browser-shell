use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum EngineCommand {
    Navigate(String),
    Resize { width: u32, height: u32, scale: f32 },
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct RenderFrame {
    pub sequence: u64,
    pub width: u32,
    pub height: u32,
    pub payload: FramePayload,
}

#[derive(Debug, Clone)]
pub enum FramePayload {
    /// Sample/in-process frame data; a real multi-process bridge should use shared memory.
    CpuBytes(Arc<[u8]>),
    /// Opaque OS shared-memory reference supplied by a platform-specific transport.
    SharedMemory {
        handle: String,
        offset: u64,
        len: u64,
    },
    /// Opaque platform GPU resource; it is not a portable `wgpu::Texture` handle.
    GpuResource { handle: String },
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("engine command channel closed")]
    CommandChannelClosed,
    #[error("frame consumer disconnected")]
    FrameConsumerDisconnected,
    #[error("engine failed: {0}")]
    Failed(String),
}

/// Runs an engine adapter on its own worker thread.
///
/// Concrete adapters own their runtime and may delegate to a separate process.
/// The UI must never call into engine code synchronously.
pub trait WebEngineAdapter: Send + 'static {
    fn run(
        self: Box<Self>,
        commands: Receiver<EngineCommand>,
        frames: Sender<RenderFrame>,
    ) -> Result<(), EngineError>;
}

pub const FRAME_INTERVAL: Duration = Duration::from_millis(8);
