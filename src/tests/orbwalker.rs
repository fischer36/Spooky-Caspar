use crate::{
    game::Game,
    input::{self, MouseButton},
};

#[cfg(test)]
mod tests {
    use crate::input::{self, MouseButton};
    use external;
    use std::ptr;
    use windows_sys::Win32::UI::{
        Input::KeyboardAndMouse::GetAsyncKeyState,
        WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN},
    };
    extern crate winapi;

    #[test]
    fn get_screen_size() {
        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

        println!("Screen width: {}", screen_width);
        println!("Screen height: {}", screen_height);
    }
    #[test]
    fn orbwalker() {
        // Assuming space key code is 0x20
        let attack_speed = 1000;
        let space_key_code = 0x20;
        let windup_delay = attack_speed * 0.16;
        // Check if the space key is being held down
        loop {
            if unsafe { GetAsyncKeyState(0x1B) } & 0x8001u16 as i16 != 0 {
                break; // Exit the loop if Escape is pressed
            }
            if unsafe { GetAsyncKeyState(space_key_code) } & 0x8001u16 as i16 != 0 {
                input::key_send('a');
                input::send_mouse_click(MouseButton::Left);
                std::thread::sleep(std::time::Duration::from_millis(windup_delay as u64 + 100));
                input::send_mouse_click(MouseButton::Right);
                std::thread::sleep(std::time::Duration::from_millis(attack_speed));
            } else {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}
