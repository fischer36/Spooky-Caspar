use std::thread;
use std::time::Duration;
use windows_sys::Win32::Foundation::POINT;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    ActivateKeyboardLayout, BlockInput, GetKeyboardLayout, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE,
    KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEINPUT,
};
use windows_sys::Win32::UI::TextServices::HKL;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetPhysicalCursorPos, SetCursorPos};

pub enum MouseButton {
    Right,
    Left,
}

pub fn send_mouse_click(mouse_button: MouseButton) -> bool {
    let (mouse_button_down, mouse_button_up) = match mouse_button {
        MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
    };
    let mouse_input_down = MOUSEINPUT {
        dx: 0,
        dy: 0,
        mouseData: 0,
        dwFlags: mouse_button_down,
        time: 0,
        dwExtraInfo: 0,
    };

    let mouse_input_up = MOUSEINPUT {
        dx: 0,
        dy: 0,
        mouseData: 0,
        dwFlags: mouse_button_up,
        time: 0,
        dwExtraInfo: 0,
    };

    let inputs = [
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 { mi: mouse_input_down },
        },
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 { mi: mouse_input_up },
        },
    ];

    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };

    sent == inputs.len() as u32
}

pub fn cursor_move(x: f32, y: f32) {
    unsafe {
        SetCursorPos(x as i32, y as i32);
    }
}

fn map_keys_to_scan_codes(key: char) -> Option<u16> {
    match key {
        'Q' | 'q' => Some(0x10),
        'W' | 'w' => Some(0x11),
        'E' | 'e' => Some(0x12), 
        'R' | 'r' => Some(0x13),
        'T' | 't' => Some(0x14),
        'A' | 'a' => Some(0x1E), 
        'K' | 'k' => Some(0x25), 
        'X' | 'x' => Some(0x2D), 
        _ => None,
    }
}
pub fn hold_key(key: char, duration_from_secs_f32: f32) {
    std::thread::spawn(move || {
        let key_scan_code = match map_keys_to_scan_codes(key) {
            Some(code) => code,
            None => {
                println!("Key not supported");
                return;
            }
        };

        let mut key_input: INPUT = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: 0,
                    wScan: key_scan_code,
                    dwFlags: KEYEVENTF_SCANCODE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let mut key_input_up = key_input.clone();
        key_input_up.Anonymous.ki.dwFlags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;

        unsafe {
            SendInput(1, &mut key_input, std::mem::size_of::<INPUT>() as i32);
        }

        std::thread::sleep(std::time::Duration::from_secs_f32(duration_from_secs_f32));

        unsafe {
            SendInput(1, &mut key_input_up, std::mem::size_of::<INPUT>() as i32);
        }
    });
}

pub fn key_down(key: char) {
    let key_scan_code = match map_keys_to_scan_codes(key) {
        Some(code) => code,
        None => {
            println!("Key not supported");
            return;
        }
    };

    let mut key_input: INPUT = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: key_scan_code,
                dwFlags: KEYEVENTF_SCANCODE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(1, &mut key_input, std::mem::size_of::<INPUT>() as i32);
    }
}

pub fn key_up(key: char) {
    let key_scan_code = match map_keys_to_scan_codes(key) {
        Some(code) => code,
        None => {
            println!("Key not supported");
            return;
        }
    };

    let mut key_input: INPUT = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: key_scan_code,
                dwFlags: KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(1, &mut key_input, std::mem::size_of::<INPUT>() as i32);
    }
}
pub fn key_send(key: char) {
    let key_scan_code = map_keys_to_scan_codes(key).unwrap();
    let current_layout: HKL = unsafe { GetKeyboardLayout(0) };
    unsafe {
        ActivateKeyboardLayout(current_layout, 0);
    }

    let mut key_input: INPUT = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: key_scan_code,
                dwFlags: KEYEVENTF_SCANCODE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let mut key_input_up = key_input.clone();
    key_input_up.Anonymous.ki.dwFlags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;

    unsafe {
        SendInput(1, &mut key_input, std::mem::size_of::<INPUT>() as i32);
    }
    thread::sleep(Duration::from_millis(5));
    unsafe {
        SendInput(1, &mut key_input_up, std::mem::size_of::<INPUT>() as i32);
    }
}

pub fn get_cursor_pos() -> nc::na::Point2<f32> {
    let mut point = POINT { x: 0, y: 0 };
    unsafe {
        GetPhysicalCursorPos(&mut point);
    }
    crate::point2!(point.x as f32, point.y as f32)
}

pub fn block_input(bool: bool) {
    match bool {
        true => unsafe { BlockInput(1) },
        false => unsafe { BlockInput(0) },
    };
}

pub fn scale_cursor_position(original_pos: (i32, i32), scale_factor: f32) -> (i32, i32) {
    let scaled_x = original_pos.0 as f32 * scale_factor;
    let scaled_y = original_pos.1 as f32 * scale_factor;

    (scaled_x.round() as i32, scaled_y.round() as i32)
}
