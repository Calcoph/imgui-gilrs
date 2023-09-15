use std::collections::HashMap;

use gilrs::GamepadId;
use imgui::{BackendFlags, Io, Key};

fn to_imgui_gamepad_key(button: gilrs::Button) -> Option<Key> {
    match button {
        gilrs::Button::South => Some(Key::GamepadFaceDown),
        gilrs::Button::East => Some(Key::GamepadFaceRight),
        gilrs::Button::North => Some(Key::GamepadFaceUp),
        gilrs::Button::West => Some(Key::GamepadFaceLeft),
        gilrs::Button::C => None,
        gilrs::Button::Z => None,
        gilrs::Button::LeftTrigger => Some(Key::GamepadL1),
        gilrs::Button::LeftTrigger2 => Some(Key::GamepadL2),
        gilrs::Button::RightTrigger => Some(Key::GamepadR1),
        gilrs::Button::RightTrigger2 => Some(Key::GamepadR1),
        gilrs::Button::Select => Some(Key::GamepadBack),
        gilrs::Button::Start => Some(Key::GamepadStart),
        gilrs::Button::Mode => None,
        gilrs::Button::LeftThumb => Some(Key::GamepadL3),
        gilrs::Button::RightThumb => Some(Key::GamepadR3),
        gilrs::Button::DPadUp => Some(Key::GamepadDpadUp),
        gilrs::Button::DPadDown => Some(Key::GamepadDpadDown),
        gilrs::Button::DPadLeft => Some(Key::GamepadDpadLeft),
        gilrs::Button::DPadRight => Some(Key::GamepadDpadRight),
        gilrs::Button::Unknown => None,
    }
}

#[derive(Clone, Copy)]
enum AnalogueGamepadInput {
    L2,
    R2,
    LUp,
    LDown,
    LLeft,
    LRight,
    RUp,
    RDown,
    RLeft,
    RRight,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

#[derive(Debug)]
struct GamepadState {
    l2: f32,
    r2: f32,
    lx: f32,
    ly: f32,
    rx: f32,
    ry: f32,
    dpad_x: f32,
    dpad_y: f32,
}

impl GamepadState {
    fn new() -> GamepadState {
        GamepadState {
            l2: 0.0,
            r2: 0.0,
            lx: 0.0,
            ly: 0.0,
            rx: 0.0,
            ry: 0.0,
            dpad_x: 0.0,
            dpad_y: 0.0,
        }
    }

    fn is_triggered(&self, trigger: AnalogueGamepadInput) -> bool {
        use AnalogueGamepadInput as Input;
        const TRESHOLD: f32 = 0.01;
        match trigger {
            Input::L2 => self.l2 > TRESHOLD,
            Input::R2 => self.r2 > TRESHOLD,
            Input::LUp => self.ly > TRESHOLD,
            Input::LDown => self.ly < -TRESHOLD,
            Input::LLeft => self.lx < -TRESHOLD,
            Input::LRight => self.lx > TRESHOLD,
            Input::RUp => self.ry > TRESHOLD,
            Input::RDown => self.ry < -TRESHOLD,
            Input::RLeft => self.rx < -TRESHOLD,
            Input::RRight => self.rx > TRESHOLD,
            Input::DPadUp => self.dpad_y > TRESHOLD,
            Input::DPadDown => self.dpad_y < -TRESHOLD,
            Input::DPadLeft => self.dpad_x > TRESHOLD,
            Input::DPadRight => self.dpad_x < -TRESHOLD,
        }
    }

    fn change_analogue_button(&mut self, io: &mut Io, button: gilrs::Button, value: f32) {
        let (analogue_input, imgui_key) = match button {
            gilrs::Button::LeftTrigger2 => (
                AnalogueGamepadInput::L2,
                to_imgui_gamepad_key(button).unwrap(),
            ),
            gilrs::Button::RightTrigger2 => (
                AnalogueGamepadInput::R2,
                to_imgui_gamepad_key(button).unwrap(),
            ),
            _ => return, // Only supports analogue bottom triggers
        };

        let was_triggered = self.is_triggered(analogue_input);
        // update state
        match analogue_input {
            AnalogueGamepadInput::L2 => self.l2 = value,
            AnalogueGamepadInput::R2 => self.r2 = value,
            _ => unreachable!(),
        }
        let is_triggered = self.is_triggered(analogue_input);
        if !was_triggered && is_triggered {
            io.add_key_event(imgui_key, true)
        } else if was_triggered && !is_triggered {
            io.add_key_event(imgui_key, false)
        }
    }

    fn change_axis(&mut self, io: &mut Io, axis: gilrs::Axis, value: f32) {
        use AnalogueGamepadInput as Input;
        let (analogue_input_neg, analogue_input_pos, imgui_key_neg, imgui_key_pos) = match axis {
            gilrs::Axis::LeftStickX => (
                Input::LLeft,
                Input::LRight,
                Key::GamepadLStickLeft,
                Key::GamepadLStickRight,
            ),
            gilrs::Axis::LeftStickY => (
                Input::LDown,
                Input::LUp,
                Key::GamepadLStickDown,
                Key::GamepadLStickUp,
            ),
            gilrs::Axis::RightStickX => (
                Input::RLeft,
                Input::RRight,
                Key::GamepadRStickLeft,
                Key::GamepadRStickRight,
            ),
            gilrs::Axis::RightStickY => (
                Input::RDown,
                Input::RUp,
                Key::GamepadRStickDown,
                Key::GamepadRStickUp,
            ),
            gilrs::Axis::DPadX => (
                Input::DPadLeft,
                Input::DPadRight,
                Key::GamepadDpadLeft,
                Key::GamepadDpadRight,
            ),
            gilrs::Axis::DPadY => (
                Input::DPadDown,
                Input::DPadUp,
                Key::GamepadDpadDown,
                Key::GamepadDpadUp,
            ),
            _ => return, // Only supports analogue L/RStick and DPad
        };

        let was_triggered_neg = self.is_triggered(analogue_input_neg);
        let was_triggered_pos = self.is_triggered(analogue_input_pos);
        // update state
        match analogue_input_neg {
            Input::LDown => self.ly = value,
            Input::LLeft => self.lx = value,
            Input::RDown => self.ry = value,
            Input::RLeft => self.rx = value,
            Input::DPadDown => self.dpad_x = value,
            Input::DPadLeft => self.dpad_y = value,
            _ => unreachable!(),
        }
        let is_triggered_neg = self.is_triggered(analogue_input_neg);
        let is_triggered_pos = self.is_triggered(analogue_input_pos);

        if !was_triggered_neg && is_triggered_neg {
            io.add_key_event(imgui_key_neg, true)
        } else if was_triggered_neg && !is_triggered_neg {
            io.add_key_event(imgui_key_neg, false)
        }

        if !was_triggered_pos && is_triggered_pos {
            io.add_key_event(imgui_key_pos, true)
        } else if was_triggered_pos && !is_triggered_pos {
            io.add_key_event(imgui_key_pos, false)
        }
    }
}

#[derive(Debug)]
pub struct GamepadHandler {
    connected_controllers: HashMap<GamepadId, GamepadState>,
}

impl Default for GamepadHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GamepadHandler {
    pub fn new() -> GamepadHandler {
        GamepadHandler {
            connected_controllers: HashMap::new(),
        }
    }

    pub fn handle_event(&mut self, io: &mut Io, controller_event: &gilrs::Event) {
        use gilrs::EventType as GEvent;
        match controller_event.event {
            GEvent::ButtonPressed(button, _) => {
                if let Some(key) = to_imgui_gamepad_key(button) {
                    io.add_key_event(key, true)
                }
            }
            GEvent::ButtonReleased(button, _) => {
                if let Some(key) = to_imgui_gamepad_key(button) {
                    io.add_key_event(key, false)
                }
            }
            GEvent::Connected => {
                self.connected_controllers
                    .insert(controller_event.id, GamepadState::new());
                io.backend_flags.insert(BackendFlags::HAS_GAMEPAD);
            }
            GEvent::Disconnected => {
                self.connected_controllers.remove(&controller_event.id);
                if self.connected_controllers.is_empty() {
                    // No connected gamepads remain
                    io.backend_flags.remove(BackendFlags::HAS_GAMEPAD);
                }
            }
            GEvent::ButtonChanged(button, value, _) => {
                if let Some(gamepad) = self.connected_controllers.get_mut(&controller_event.id) {
                    gamepad.change_analogue_button(io, button, value);
                }
            }
            GEvent::AxisChanged(axis, value, _) => {
                if let Some(gamepad) = self.connected_controllers.get_mut(&controller_event.id) {
                    gamepad.change_axis(io, axis, value);
                }
            }
            GEvent::ButtonRepeated(_, _) => (),
            GEvent::Dropped => (),
        }
    }
}
