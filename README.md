# Rust browser shell scaffold

This workspace separates a native shell from a replaceable web-engine adapter:

- `web-engine-api` defines commands, render-frame metadata, and the
  `WebEngineAdapter` boundary.
- `browser-shell` demonstrates a dedicated engine worker, a Tokio-based IPC
  forwarding loop, and a bounded 120 Hz UI-loop stand-in.

Run the scaffold with `cargo run -p browser-shell`. The included mock engine
emits empty frames; it is plumbing, not a browser renderer.

## Backend and GPU integration

Add a concrete `WebEngineAdapter` for Servo, CEF, or another engine. Keep engine
startup, JavaScript execution, networking, and layout/compositing off the shell
UI thread. `wgpu` is an optional dependency enabled with `--features gpu` for a
future shell renderer. A production backend should negotiate a platform-specific
shared-memory or external-GPU-resource transport; a `wgpu::Texture` is not a
portable cross-process texture handle. If interop is unavailable, copy pixels
through shared memory and upload them on the UI renderer's GPU device.

## Thread-isolation guidelines

- Treat engine commands and frame delivery as asynchronous messages. Never wait
  synchronously for navigation, JavaScript, layout, or a frame on the UI thread.
- Use bounded queues and discard/coalesce stale render frames. This sample keeps
  a single pending UI frame so a slow presenter does not accumulate latency.
- Keep input handling and presentation on the native UI/event-loop thread; send
  input to the engine asynchronously and use the latest completed frame.
- For fault isolation, run the engine in a separate process with a versioned IPC
  protocol, explicit timeouts, crash recovery, and OS-level resource limits.
- `wry` webviews commonly have platform event-loop/thread affinity. Treat Wry as
  an in-process integration with those constraints, not as a generic engine to
  construct on any worker thread. A CEF/Servo process adapter is a better fit
  when strict UI/engine process isolation is required.
- If a backend exposes GPU frames, negotiate compatible devices, formats,
  synchronization primitives, and resource lifetimes. Import/export is
  platform-specific; otherwise fall back to shared-memory pixels.

## Dependency choices

The workspace includes Tokio for async IPC orchestration, crossbeam channels
for bounded worker communication, `anyhow` for shell-level context, and
`thiserror` for typed adapter errors. `wgpu` is optional so the scaffold runs
without committing to a renderer implementation. Add a platform-specific Wry
or CEF dependency only alongside a concrete adapter and its event-loop/process
integration.
