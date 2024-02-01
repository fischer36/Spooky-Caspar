use log::{info, warn};
use windows_sys::Win32::Foundation::{HMODULE, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_ESCAPE, VK_HOME, VK_INSERT, VK_M, VK_RSHIFT, VK_SPACE, VK_TAB, VK_U,
};
use windows_sys::Win32::UI::WindowsAndMessaging::WH_KEYBOARD_LL;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowTextW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, HHOOK, MSG, WM_KEYDOWN,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, WM_KEYUP};

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;

use crate::MainLoopThreadCommunication;

pub enum KbHookMessage {
    Keypress(char),
    VKPress(VIRTUAL_KEY),
    SpaceIsDown(bool),
    QIsDown(bool),
    WIsDown(bool),
    EIsDown(bool),
    RIsDown(bool),
}

thread_local! {
    static SENDER: RefCell<Option<Sender<MainLoopThreadCommunication>>> = RefCell::new(None);
}

pub static mut HOOK_HANDLE: HHOOK = 0 as HHOOK;
static mut SPACE_PRESSED: bool = false;

unsafe extern "system" fn hook_callback(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb_struct = &*(l_param as *const KBDLLHOOKSTRUCT);

        match kb_struct.vkCode as u16 {
            VK_SPACE => {
                if check_window() {
                    if w_param == WM_KEYDOWN as WPARAM && SPACE_PRESSED == false {
                        // Space key press
                        space_key(true);
                        SPACE_PRESSED = true;
                    } else if w_param == WM_KEYUP as WPARAM && SPACE_PRESSED == true {
                        // Space key release
                        space_key(false);
                        SPACE_PRESSED = false;
                    }
                }
            }
            VK_END => {
                if w_param == WM_KEYDOWN as WPARAM {
                    exit_pressed();
                }
            }
            VK_ESCAPE => {
                if w_param == WM_KEYDOWN as WPARAM {
                    vk_press(VK_ESCAPE);
                }
            }
            VK_U => {
                if w_param == WM_KEYDOWN as WPARAM {
                    u_key_press();
                }
            }
            VK_M => {
                if w_param == WM_KEYDOWN as WPARAM {
                    m_key_press();
                }
            }

            _ => (),
        }
    }
    CallNextHookEx(HOOK_HANDLE, code, w_param, l_param)
}
unsafe fn vk_press(vk: VIRTUAL_KEY) {
    SENDER.with(|global_sender| {
        if let Some(sender) = &*global_sender.borrow() {
            sender
                .send(MainLoopThreadCommunication::KbHookThread(KbHookMessage::VKPress(vk)))
                .unwrap();
        }
    });
    std::thread::sleep(Duration::from_millis(200))
}
unsafe fn char_key_press(c: char) {
    SENDER.with(|global_sender| {
        if let Some(sender) = &*global_sender.borrow() {
            sender
                .send(MainLoopThreadCommunication::KbHookThread(KbHookMessage::Keypress(c)))
                .unwrap();
        }
    });
    std::thread::sleep(Duration::from_millis(200))
}
unsafe fn m_key_press() {
    SENDER.with(|global_sender| {
        if let Some(sender) = &*global_sender.borrow() {
            sender
                .send(MainLoopThreadCommunication::KbHookThread(KbHookMessage::Keypress('m')))
                .unwrap();
        }
    });
    std::thread::sleep(Duration::from_millis(200));
}

unsafe fn u_key_press() {
    SENDER.with(|global_sender| {
        if let Some(sender) = &*global_sender.borrow() {
            sender
                .send(MainLoopThreadCommunication::KbHookThread(KbHookMessage::Keypress('u')))
                .unwrap();
        }
    });
    std::thread::sleep(Duration::from_millis(200));
}

unsafe fn space_key(state: bool) {
    SENDER.with(|global_sender| {
        if let Some(sender) = &*global_sender.borrow() {
            sender
                .send(MainLoopThreadCommunication::KbHookThread(KbHookMessage::SpaceIsDown(
                    state,
                )))
                .unwrap();
        }
    });
}

unsafe fn exit_pressed() {
    SENDER.with(|global_sender| {
        if let Some(sender) = &*global_sender.borrow() {
            sender
                .send(MainLoopThreadCommunication::Exit(Some(
                    "Exit key was pressed".to_string(),
                )))
                .unwrap();
        }
    });
}

// This function now takes an Arc<AtomicBool> to set the flag.
pub fn set_keyboard_hook_and_run_message_loop(
    hook_set_flag: Arc<AtomicBool>,
    sender: Sender<MainLoopThreadCommunication>,
) {
    info!("Trying to set keyboard hook");
    unsafe {
        HOOK_HANDLE = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_callback), HMODULE::default(), 0);
        if HOOK_HANDLE == 0 {
            warn!("Failed to set keyboard hook");
            hook_set_flag.store(false, Ordering::SeqCst); // Hook was not set
            return;
        } else {
            hook_set_flag.store(true, Ordering::SeqCst); // Hook was set successfully
            info!("Keyboard hook set successfully");
        }
        SENDER.with(|global_sender| {
            *global_sender.borrow_mut() = Some(sender);
        });
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0 as _, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

pub fn remove_keyboard_hook() {
    info!("Removing keyboard hook");
    unsafe {
        if HOOK_HANDLE != 0 {
            UnhookWindowsHookEx(HOOK_HANDLE);
            HOOK_HANDLE = 0;
        }
    }
    std::process::exit(0);
}

pub unsafe fn check_window() -> bool {
    let foreground_window = GetForegroundWindow();
    let mut window_title = [0u16; 30];
    GetWindowTextW(foreground_window, window_title.as_mut_ptr(), window_title.len() as i32);
    let foreground_window_title = String::from_utf16_lossy(&window_title[..]);
    return foreground_window_title.starts_with("League of Legends (TM)");
}
