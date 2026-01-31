// --- START OF FILE src/action_executor.rs ---
use windows::core::PWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_SHIFT, VK_MENU, VK_LWIN, VK_ESCAPE, VK_TAB,
    VK_RETURN, VK_BACK, VK_SPACE,
    VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12,
    VK_DELETE, VK_HOME, VK_END, VK_PRIOR, VK_NEXT,
    VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, PostMessageW, WM_APPCOMMAND,
};

#[derive(Debug, Clone)]
pub enum Action {
    KeyCombo(String),
    Run(String),
    AppCommand(u32), // New variant for APPCOMMANDs
}

pub fn execute_action(action: &Action) {
    match action {
        Action::KeyCombo(combo) => {
            send_key_combo(combo);
        }
        Action::Run(path) => {
            launch_program(path);
        }
        Action::AppCommand(cmd) => {
            send_app_command(*cmd);
        }
    }
}

fn send_key_combo(combo: &str) {
    let parts: Vec<&str> = combo.split('+').map(|s| s.trim()).collect();
    
    let mut modifiers = Vec::new();
    let mut main_key = None;

    for part in &parts {
        match part.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => modifiers.push(VK_CONTROL),
            "SHIFT" => modifiers.push(VK_SHIFT),
            "ALT" | "MENU" => modifiers.push(VK_MENU),
            "WIN" | "GUI" => modifiers.push(VK_LWIN),
            key => main_key = Some(parse_key(key)),
        }
    }

    unsafe {
        // Press modifiers
        for &modifier in &modifiers {
            send_key(modifier, false);
        }

        // Press main key
        if let Some(key) = main_key {
            send_key(key, false);
            // Removed std::thread::sleep here - SendInput should be fast enough.
            send_key(key, true);
        }

        // Release modifiers
        for &modifier in modifiers.iter().rev() {
            send_key(modifier, true);
        }
    }
}

fn parse_key(key: &str) -> VIRTUAL_KEY {
    match key {
        // Special keys
        "ESC" | "ESCAPE" => VK_ESCAPE,
        "TAB" => VK_TAB,
        "ENTER" | "RETURN" => VK_RETURN,
        "BACKSPACE" => VK_BACK,
        "SPACE" => VK_SPACE,
        "DELETE" => VK_DELETE,
        
        // Navigation
        "HOME" => VK_HOME,
        "END" => VK_END,
        "PAGE_UP" | "PAGEUP" => VK_PRIOR,
        "PAGE_DOWN" | "PAGEDOWN" => VK_NEXT,
        "LEFT_ARROW" | "LEFT" => VK_LEFT,
        "RIGHT_ARROW" | "RIGHT" => VK_RIGHT,
        "UP_ARROW" | "UP" => VK_UP,
        "DOWN_ARROW" | "DOWN" => VK_DOWN,
        
        // Function keys
        "F1" => VK_F1,
        "F2" => VK_F2,
        "F3" => VK_F3,
        "F4" => VK_F4,
        "F5" => VK_F5,
        "F6" => VK_F6,
        "F7" => VK_F7,
        "F8" => VK_F8,
        "F9" => VK_F9,
        "F10" => VK_F10,
        "F11" => VK_F11,
        "F12" => VK_F12,
        
        // Media keys (these are also mapped from string names)
        "BRIGHTNESS_DOWN" => VIRTUAL_KEY(0xE6), // Correct VK for brightness down
        "BRIGHTNESS_UP" => VIRTUAL_KEY(0xE7),   // Correct VK for brightness up
        "MEDIA_NEXT" | "NEXT_TRACK" => VIRTUAL_KEY(0xB0),
        "MEDIA_PREV" | "PREV_TRACK" => VIRTUAL_KEY(0xB1),
        "MEDIA_PLAY_PAUSE" | "PLAY_PAUSE" => VIRTUAL_KEY(0xB3),
        "MEDIA_STOP" => VIRTUAL_KEY(0xB2),
        "MUTE" | "VOLUME_MUTE" => VIRTUAL_KEY(0xAD),
        "VOLUME_DOWN" => VIRTUAL_KEY(0xAE),
        "VOLUME_UP" => VIRTUAL_KEY(0xAF),
        
        // Numbers
        "0" => VIRTUAL_KEY(0x30),
        "1" => VIRTUAL_KEY(0x31),
        "2" => VIRTUAL_KEY(0x32),
        "3" => VIRTUAL_KEY(0x33),
        "4" => VIRTUAL_KEY(0x34),
        "5" => VIRTUAL_KEY(0x35),
        "6" => VIRTUAL_KEY(0x36),
        "7" => VIRTUAL_KEY(0x37),
        "8" => VIRTUAL_KEY(0x38),
        "9" => VIRTUAL_KEY(0x39),
        
        // Letters
        "A" => VIRTUAL_KEY(0x41),
        "B" => VIRTUAL_KEY(0x42),
        "C" => VIRTUAL_KEY(0x43),
        "D" => VIRTUAL_KEY(0x44),
        "E" => VIRTUAL_KEY(0x45),
        "F" => VIRTUAL_KEY(0x46),
        "G" => VIRTUAL_KEY(0x47),
        "H" => VIRTUAL_KEY(0x48),
        "I" => VIRTUAL_KEY(0x49),
        "J" => VIRTUAL_KEY(0x4A),
        "K" => VIRTUAL_KEY(0x4B),
        "L" => VIRTUAL_KEY(0x4C),
        "M" => VIRTUAL_KEY(0x4D),
        "N" => VIRTUAL_KEY(0x4E),
        "O" => VIRTUAL_KEY(0x4F),
        "P" => VIRTUAL_KEY(0x50),
        "Q" => VIRTUAL_KEY(0x51),
        "R" => VIRTUAL_KEY(0x52),
        "S" => VIRTUAL_KEY(0x53),
        "T" => VIRTUAL_KEY(0x54),
        "U" => VIRTUAL_KEY(0x55),
        "V" => VIRTUAL_KEY(0x56),
        "W" => VIRTUAL_KEY(0x57),
        "X" => VIRTUAL_KEY(0x58),
        "Y" => VIRTUAL_KEY(0x59),
        "Z" => VIRTUAL_KEY(0x5A),
        
        // Symbols (OEM keys - these work for US keyboard layout)
        "MINUS" | "-" | "_" => VIRTUAL_KEY(0xBD),
        "EQUALS" | "=" | "+" => VIRTUAL_KEY(0xBB),
        "LEFT_BRACKET" | "LBRACKET" | "[" | "{" => VIRTUAL_KEY(0xDB),
        "RIGHT_BRACKET" | "RBRACKET" | "]" | "}" => VIRTUAL_KEY(0xDD),
        "SEMICOLON" | ";" | ":" => VIRTUAL_KEY(0xBA),
        "APOSTROPHE" | "'" | "\"" => VIRTUAL_KEY(0xDE),
        "GRAVE" | "`" | "~" => VIRTUAL_KEY(0xC0),
        "BACKSLASH" | "\\" | "|" => VIRTUAL_KEY(0xDC),
        "COMMA" | "," | "<" => VIRTUAL_KEY(0xBC),
        "PERIOD" | "." | ">" => VIRTUAL_KEY(0xBE),
        "SLASH" | "/" | "?" => VIRTUAL_KEY(0xBF),
        
        _ => {
            eprintln!("Unknown key: {}", key);
            VIRTUAL_KEY(0)
        }
    }
}

unsafe fn send_key(vk: VIRTUAL_KEY, is_up: bool) {
    if vk.0 == 0 {
        return; // Skip invalid keys
    }
    
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if is_up { KEYEVENTF_KEYUP } else { Default::default() },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
}

fn send_app_command(app_cmd: u32) {
    unsafe {
        let hwnd_fg = GetForegroundWindow();
        if hwnd_fg.0 != 0 {
            // WM_APPCOMMAND takes app command in HIWORD(lParam)
            // and the target device (keyboard/mouse) in LOWORD(lParam)
            // Here we indicate the command came from a keyboard.
            let lparam: isize = (app_cmd as isize) << 16;
            let result = PostMessageW(hwnd_fg, WM_APPCOMMAND, 0, lparam);
            if result.is_ok() {
                println!("✓ Sent APPCOMMAND: {}", app_cmd);
            } else {
                eprintln!("✗ Failed to send APPCOMMAND {}: {:?}", app_cmd, result);
                eprintln!("  The foreground application may not support this command.");
            }
        } else {
            eprintln!("✗ No foreground window to send APPCOMMAND {}", app_cmd);
            eprintln!("  Hint: Focus a window before triggering this command.");
        }
    }
}


fn launch_program(path: &str) {
    unsafe {
        let mut cmd_line = widestring(path);
        
        let mut si = STARTUPINFOW::default();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        
        let mut pi = PROCESS_INFORMATION::default();

        match CreateProcessW(
            None,
            PWSTR(cmd_line.as_mut_ptr()),
            None,
            None,
            false,
            Default::default(),
            None,
            None,
            &si,
            &mut pi,
        ) {
            Ok(_) => {
                println!("✓ Successfully launched: {}", path);
                // Close handles to avoid leaks
                let _ = CloseHandle(pi.hProcess);
                let _ = CloseHandle(pi.hThread);
            }
            Err(e) => {
                eprintln!("✗ Failed to launch '{}': {}", path, e);
                eprintln!("  Error code: {:?}", e.code());
                eprintln!("  Hint: Ensure the program path is correct and accessible.");
            }
        }
    }
}

fn widestring(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}