Crate to send gilrs gamepad events to imgui for gamepad navigation.

## Usage
Create a `GamepadHandler` using `GamepadHandler::new()` before the main loop.

Call `GamepadHandler::handle_event()` for every gilrs event (or every winit event if `winit` feature is enabled).

Make sure to enable gamepad navigation on your imgui application:
```rust
let io = Context::io_mut();
io.config_flags |= imgui::ConfigFlags::NAV_ENABLE_GAMEPAD;
```

See [Troubleshooting](#Troubleshooting) if encountering any issue.

## Features
* `winit`: allows `GamepadHandler::handle_event()` to also call `WinitPlatform::handle_event()`

## Troubleshooting
If using the `imgui-wgpu` crate, and the program crashes when opening the window menu (hold X on XBOX or Square on PlayStation) then you must use commit `89394e0` or later of the crate. You can do that by inserting the following in your Cargo.toml:

```toml
[patch.crates-io]
imgui-wgpu = { git = "https://github.com/Yatekii/imgui-wgpu-rs", rev = "89394e0" }
```
