Crate to send gilrs gamepad events to imgui for gamepad navigation.

## Usage
Create a `GamepadHandler` using `GamepadHandler::new()` before the main loop.

Call `GamepadHandler::handle_event()` for every gilrs event (or every winit event if `winit` feature is enabled).

Make sure to enable gamepad navigation on your imgui application:
```rust
let io = Context::io_mut();
io.config_flags |= imgui::ConfigFlags::NAV_ENABLE_GAMEPAD;
```

## Features
* `winit`: allows `GamepadHandler::handle_event()` to also call `WinitPlatform::handle_event()`
