# Multi-output TTY acceptance

Use this checklist for a hands-on check of scrolling tiling on real outputs. It
does not depend on a machine-specific launcher or config. Run driftline from a
separate TTY with a test config and session file so the normal desktop state is
left alone. Build the current checkout first:

```bash
cargo build --release --locked
./target/release/driftline --backend udev --config /path/to/test-config.toml --session-file /path/to/test-session.json
```

Omit `--drm-connector` to include every connected output. Use the actual
connector name with `--drm-connector HDMI-A-1` to limit the run to one output.
The config can use the normal defaults, or assign temporary test bindings under
`[keybindings]`. See the [user guide](../../docs/tiling.md) for defaults and
[configuration reference](../../docs/config.md) for options. Exit with
`Mod+Ctrl+Shift+Q` unless the test config changes that binding.

## Multi-output checklist

Record each output's resolution and scale, then check:

1. Pan the output views apart. Open a window on each output and press
   `Mod+Ctrl+T`. Each window should join its own output's strip.
2. Move the pointer between outputs. Focus, reorder, stack and resize columns
   on one output; confirm the other output keeps its selection, width and
   order.
3. Open windows on both outputs. Confirm each joins the output showing its
   canvas-space center. Where views overlap, the pointer-focused output owns
   the new window.
4. Transfer a tiled window with `Mod+Alt+Arrow`. Confirm the destination gets
   focus and the window remains tiled. Disconnect one output and confirm its
   columns remain available on the surviving output. Reconnect it and confirm
   it gets a fresh empty layout.
5. Switch to canvas. Confirm windows return to their saved positions and that
   no window is lost, stuck mid-configure or left displaying stale content.

Record the observations and any failure in
[`tiling-status.md`](tiling-status.md). Automated fixture and GPU tests cover
many protocol and rendering transitions, but do not substitute for testing the
actual hardware, driver and monitor combination.
