#[cfg(test)]
mod test_offsets;
use crate::io::output::{self, MouseButton};
use external;
use std::ptr;
use windows_sys::Win32::UI::{
    Input::KeyboardAndMouse::GetAsyncKeyState,
    WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN},
};

#[test]
fn get_screen_size() {
    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

    println!("Screen width: {}", screen_width);
    println!("Screen height: {}", screen_height);
}
