use controller_emulator::controller::ns_procon;
use controller_emulator::controller::ns_procon::inputs;
use controller_emulator::controller::Controller;
use controller_emulator::usb_gadget;
use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs};

fn get_button_input(button: Button) -> Option<usize> {
    match button {
        Button::South => Some(inputs::BUTTON_B),
        Button::East => Some(inputs::BUTTON_A),
        Button::North => Some(inputs::BUTTON_X),
        Button::West => Some(inputs::BUTTON_Y),
        Button::LeftTrigger => Some(inputs::BUTTON_L),
        Button::LeftTrigger2 => Some(inputs::BUTTON_ZL),
        Button::RightTrigger => Some(inputs::BUTTON_R),
        Button::RightTrigger2 => Some(inputs::BUTTON_ZR),
        Button::Select => Some(inputs::BUTTON_MINUS),
        Button::Start => Some(inputs::BUTTON_PLUS),
        Button::Mode => Some(inputs::BUTTON_HOME),
        Button::LeftThumb => Some(inputs::BUTTON_L_STICK),
        Button::RightThumb => Some(inputs::BUTTON_R_STICK),
        Button::DPadUp => Some(inputs::BUTTON_UP),
        Button::DPadDown => Some(inputs::BUTTON_DOWN),
        Button::DPadLeft => Some(inputs::BUTTON_LEFT),
        Button::DPadRight => Some(inputs::BUTTON_RIGHT),
        _ => None,
    }
}

fn get_axis_input(axis: Axis) -> Option<usize> {
    match axis {
        Axis::LeftStickX => Some(inputs::AXIS_LH),
        Axis::LeftStickY => Some(inputs::AXIS_LV),
        Axis::RightStickX => Some(inputs::AXIS_RH),
        Axis::RightStickY => Some(inputs::AXIS_RV),
        _ => None,
    }
}

fn next_connected(gilrs: &Gilrs) -> Option<GamepadId> {
    for (id, _gamepad) in gilrs.gamepads() {
        return Some(id);
    }
    return None;
}

fn find_and_set_active(
    active_gamepad: &mut Option<GamepadId>,
    procon: &mut Option<ns_procon::NsProcon>,
    gilrs: &Gilrs,
) {
    *active_gamepad = next_connected(gilrs);

    if let Some(active_gamepad) = active_gamepad {
        let mut new_procon = ns_procon::NsProcon::create("/dev/hidg0", [255, 0, 0]);
        let _ = new_procon.start_comms();
        *procon = Some(new_procon);
        println!(
            "Gamepad is now active: {}",
            gilrs.gamepad(*active_gamepad).name()
        )
    } else {
        if let Some(procon) = procon {
            procon.stop();
        }
        *procon = None;
        println!("No gamepad active")
    }
}

fn main() {
    let mut gilrs = Gilrs::new().unwrap();
    usb_gadget::reset("procons");

    let mut active_gamepad = None;
    let mut procon = None;
    find_and_set_active(&mut active_gamepad, &mut procon, &gilrs);

    loop {
        while let Some(Event { id, event, time: _ }) = gilrs.next_event() {
            match event {
                EventType::Connected => {
                    if let Some(connected) = gilrs.connected_gamepad(id) {
                        println!("New gamepad connected: {}", connected.name());

                        find_and_set_active(&mut active_gamepad, &mut procon, &gilrs);
                    }
                }
                EventType::Disconnected => {
                    let gamepad = gilrs.gamepad(id);
                    println!("Gamepad disconnected: {}", gamepad.name());

                    if active_gamepad == Some(id) {
                        find_and_set_active(&mut active_gamepad, &mut procon, &gilrs);
                    }
                }
                EventType::ButtonPressed(button, _code) => {
                    if let Some(procon) = procon.as_mut() {
                        if let Some(button) = get_button_input(button) {
                            let _ = procon.press(button, true);
                        }
                    }
                }
                EventType::ButtonReleased(button, _code) => {
                    if let Some(procon) = procon.as_mut() {
                        if let Some(button) = get_button_input(button) {
                            let _ = procon.release(button, true);
                        }
                    }
                }
                EventType::AxisChanged(axis, value, _code) => {
                    if let Some(procon) = procon.as_mut() {
                        if let Some(axis) = get_axis_input(axis) {
                            println!("{:?}", value);
                            let value = ((value + 1.0) * 0.5 * 65535.0) as u16;
                            println!("{:?}", value);
                            let _ = procon.set_axis(axis, value, true);
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
