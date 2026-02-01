// --- START OF FILE src/main.rs ---
mod hid_parser;
mod key_mapper;
mod action_executor;
mod variable_maps;

use std::cell::RefCell;
use std::rc::Rc;
use std::ptr::null_mut;
use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, 
    RAWINPUTHEADER, RAWINPUTDEVICE_FLAGS, RID_INPUT, RIDEV_INPUTSINK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG, WM_DESTROY,
    WM_INPUT, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_OVERLAPPEDWINDOW,
    PostMessageW, WM_USER,
    SetWindowsHookExW, CallNextHookEx, UnhookWindowsHookEx, WH_KEYBOARD_LL, KBDLLHOOKSTRUCT,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use notify::{Watcher, RecommendedWatcher, RecursiveMode};
use notify::event::{EventKind, ModifyKind};
use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem, PredefinedMenuItem}};
use tray_icon::Icon;

use key_mapper::KeyMapper;




// Custom window messages
const WM_RELOAD_CONFIG: u32 = WM_USER + 1;
const WM_RESET_CONFIG: u32 = WM_USER + 2;
const WM_EXIT_APP: u32 = WM_USER + 3;

// Thread-local storage for the key mapper
// IMPORTANT: This assumes all HID input processing happens on the window message thread.
// The Windows raw input API guarantees WM_INPUT messages are delivered to the thread
// that created the window, so this assumption holds as long as we don't spawn additional
// threads to process HID input.
thread_local! {
    static GLOBAL_MAPPER: RefCell<Option<Rc<RefCell<KeyMapper>>>> = RefCell::new(None);
    static MAPPING_FILE_PATH: RefCell<Option<PathBuf>> = RefCell::new(None);
    static MAIN_WINDOW: RefCell<Option<HWND>> = RefCell::new(None);
    static SUPPRESSED_KEYS: RefCell<std::collections::HashSet<u32>> = RefCell::new(std::collections::HashSet::new());
    static H_HOOK: RefCell<Option<windows::Win32::UI::WindowsAndMessaging::HHOOK>> = RefCell::new(None);
}

fn main() -> windows::core::Result<()> {
    // Fail-safe startup print
    println!("--- A1314 Daemon DEBUG START (PID: {}) ---", std::process::id());

    // Initialize logging - Force DEBUG level for troubleshooting
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--install" => {
                return install_service();
            }
            "--uninstall" => {
                return uninstall_service();
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    log::info!("{} v{} starting...", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    log::info!("Log level: {} (set RUST_LOG environment variable to change)", log::max_level());

    // Force initialization of lazy_static maps
    let _ = variable_maps::STRING_TO_HID_KEY.len();
    let _ = variable_maps::STRING_TO_ACTION.len();

    // Get mapping file path
    let exe_path = std::env::current_exe()
        .expect("Failed to get executable path");
    let exe_dir = exe_path.parent()
        .expect("Failed to get executable directory");
    let mapping_path = exe_dir.join("A1314_mapping.txt");

    log::info!("Executable location: {}", exe_path.display());
    log::info!("Looking for mapping file: {}", mapping_path.display());

    // Create default mapping file if it doesn't exist
    if !mapping_path.exists() {
        log::warn!("Mapping file not found, creating default mapping file");
        create_default_mapping_file(&mapping_path)?;
    }

    // Store mapping path globally
    MAPPING_FILE_PATH.with(|path| {
        *path.borrow_mut() = Some(mapping_path.clone());
    });

    let mapper = Rc::new(RefCell::new(KeyMapper::new()));
    mapper.borrow_mut().load_mapping_file(&mapping_path);

    GLOBAL_MAPPER.with(|gm| {
        *gm.borrow_mut() = Some(mapper.clone());
    });

    unsafe {
        let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?;

        let class_name = widestring("A1314DaemonClass");
        let window_name = widestring("A1314Daemon"); // Bind to variable to avoid dangling pointer
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_name.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            hinstance,
            None,
        )?;

        // Store window handle globally
        MAIN_WINDOW.with(|wnd| {
            *wnd.borrow_mut() = Some(hwnd);
        });

        register_raw_input(hwnd)?;
        log::info!("Raw input registered successfully");

        // Install keyboard hook
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), hinstance, 0)?;
        H_HOOK.with(|h| *h.borrow_mut() = Some(hook));
        log::info!("Low-level keyboard hook installed for key suppression");

        // Create system tray icon
        if let Err(e) = create_system_tray(&exe_dir) {
            log::error!("Failed to create system tray icon: {}", e);
        } else {
            log::info!("System tray icon created");
        }

        // Start file watcher for hot reload
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(ModifyKind::Data(_))) {
                        let _ = tx.send(());
                    }
                }
            }
        ).expect("Failed to create file watcher");

        watcher.watch(&mapping_path, RecursiveMode::NonRecursive)
            .expect("Failed to watch mapping file");

        log::info!("File watcher started for hot reload");
        log::info!("Daemon is now running. Use system tray icon to control.");

        // Start a thread to handle file watch events
        let hwnd_val = hwnd.0 as usize;
        std::thread::spawn(move || {
            let hwnd = HWND(hwnd_val as *mut c_void);
            handle_file_watch_events(rx, hwnd);
        });

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(null_mut()), 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Keep watcher alive until shutdown
        drop(watcher);
    }

    log::info!("Daemon shutting down");

    // Cleanup hook
    H_HOOK.with(|h| {
        if let Some(hook) = *h.borrow() {
            unsafe {
                let _ = UnhookWindowsHookEx(hook);
            }
            log::info!("Low-level keyboard hook uninstalled");
        }
    });

    Ok(())
}

fn handle_file_watch_events(rx: Receiver<()>, hwnd: HWND) {
    while rx.recv().is_ok() {
        // Debounce: wait a bit to avoid multiple rapid reloads
        std::thread::sleep(Duration::from_millis(100));
        
        // Drain any additional events that came in during the debounce period
        while rx.try_recv().is_ok() {}
        
        log::info!("Mapping file changed, reloading...");
        unsafe {
            let _ = PostMessageW(hwnd, WM_RELOAD_CONFIG, WPARAM(0), LPARAM(0));
        }
    }
}

fn create_system_tray(_exe_dir: &std::path::Path) -> Result<(), String> {
    // Load icon from embedded resources (ordinal 1 is standard for winres)
    let icon = Icon::from_resource(1, Some((32, 32)))
        .or_else(|_| {
            log::warn!("Failed to load icon from resource, using fallback");
            Icon::from_rgba(vec![255; 32 * 32 * 4], 32, 32)
        })
        .map_err(|e| format!("Failed to create icon: {}", e))?;

    // Create menu
    let menu = Menu::new();
    
    let reload_item = MenuItem::new("Reload Configuration", true, None);
    let reset_item = MenuItem::new("Reset to Default Configuration", true, None);
    let separator1 = PredefinedMenuItem::separator();
    let exit_item = MenuItem::new("Exit", true, None);

    menu.append(&reload_item).map_err(|e| format!("Menu error: {}", e))?;
    menu.append(&reset_item).map_err(|e| format!("Menu error: {}", e))?;
    menu.append(&separator1).map_err(|e| format!("Menu error: {}", e))?;
    menu.append(&exit_item).map_err(|e| format!("Menu error: {}", e))?;

    // Build tray icon
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("A1314 Keyboard Daemon")
        .with_icon(icon)
        .build()
        .map_err(|e| format!("Failed to build tray icon: {}", e))?;

    // Pre-clone IDs for the thread to avoid capturing Send-hostile types
    let reload_id = reload_item.id().clone();
    let reset_id = reset_item.id().clone();
    let exit_id = exit_item.id().clone();

    // Handle menu events
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = tray_icon::menu::MenuEvent::receiver().recv() {
                MAIN_WINDOW.with(|wnd| {
                    if let Some(hwnd) = *wnd.borrow() {
                        unsafe {
                            if event.id == reload_id {
                                let _ = PostMessageW(hwnd, WM_RELOAD_CONFIG, WPARAM(0), LPARAM(0));
                            } else if event.id == reset_id {
                                let _ = PostMessageW(hwnd, WM_RESET_CONFIG, WPARAM(0), LPARAM(0));
                            } else if event.id == exit_id {
                                let _ = PostMessageW(hwnd, WM_EXIT_APP, WPARAM(0), LPARAM(0));
                            }
                        }
                    }
                });
            }
        }
    });

    // Keep tray icon alive by leaking it (it will be cleaned up on program exit)
    Box::leak(Box::new(_tray_icon));

    Ok(())
}

fn reload_configuration() {
    MAPPING_FILE_PATH.with(|path| {
        if let Some(mapping_path) = &*path.borrow() {
            GLOBAL_MAPPER.with(|gm| {
                if let Some(mapper_rc) = &*gm.borrow() {
                    log::info!("Reloading configuration from {}", mapping_path.display());
                    mapper_rc.borrow_mut().load_mapping_file(mapping_path);
                    log::info!("Configuration reloaded successfully");
                }
            });
        }
    });
}

fn reset_configuration() {
    MAPPING_FILE_PATH.with(|path| {
        if let Some(mapping_path) = &*path.borrow() {
            log::info!("Resetting configuration to defaults");
            match create_default_mapping_file(mapping_path) {
                Ok(_) => {
                    log::info!("Default configuration file created");
                    reload_configuration();
                }
                Err(e) => {
                    log::error!("Failed to reset configuration: {}", e);
                }
            }
        }
    });
}

fn create_default_mapping_file(path: &std::path::Path) -> windows::core::Result<()> {
    let default_content = include_str!("../A1314_mapping.txt");
    std::fs::write(path, default_content)
        .map_err(|e| {
            log::error!("Failed to write default mapping file: {}", e);
            windows::core::Error::from_win32()
        })?;
    log::info!("Created default mapping file at {}", path.display());
    Ok(())
}

unsafe fn register_raw_input(hwnd: HWND) -> windows::core::Result<()> {
    let devices = [
        RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x06,
            dwFlags: RAWINPUTDEVICE_FLAGS(RIDEV_INPUTSINK.0),
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: 0x0C,
            usUsage: 0x01,
            dwFlags: RAWINPUTDEVICE_FLAGS(RIDEV_INPUTSINK.0),
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: 0xFF00,
            usUsage: 0x01,
            dwFlags: RAWINPUTDEVICE_FLAGS(RIDEV_INPUTSINK.0),
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: 0xFF00,
            usUsage: 0x03, // Explicitly for some Apple Fn key implementations
            dwFlags: RAWINPUTDEVICE_FLAGS(RIDEV_INPUTSINK.0),
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: 0xFF01, // Another vendor usage page sometimes used by Apple
            usUsage: 0x01,
            dwFlags: RAWINPUTDEVICE_FLAGS(RIDEV_INPUTSINK.0),
            hwndTarget: hwnd,
        },
    ];

    RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32)?;
    Ok(())
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_INPUT => {
                handle_raw_input(lparam);
                LRESULT(0)
            }
            WM_RELOAD_CONFIG => {
                reload_configuration();
                LRESULT(0)
            }
            WM_RESET_CONFIG => {
                reset_configuration();
                LRESULT(0)
            }
            WM_EXIT_APP => {
                log::info!("Exit requested from system tray");
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_DESTROY => {
                log::info!("Received WM_DESTROY, shutting down");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

const RIM_TYPEHID: u32 = 2;
const RIM_TYPEKEYBOARD: u32 = 1;

unsafe fn handle_raw_input(lparam: LPARAM) {
    let hrawinput = HRAWINPUT(lparam.0 as *mut c_void);
    
    // First call: get the size of the RAWINPUT structure
    let mut size = 0u32;
    GetRawInputData(
        hrawinput,
        RID_INPUT,
        None,
        &mut size,
        std::mem::size_of::<RAWINPUTHEADER>() as u32,
    );

    if size == 0 {
        return;
    }

    // Second call: get the actual RAWINPUT data
    let mut buffer = vec![0u8; size as usize];
    let res = GetRawInputData(
        hrawinput,
        RID_INPUT,
        Some(buffer.as_mut_ptr() as *mut c_void),
        &mut size,
        std::mem::size_of::<RAWINPUTHEADER>() as u32,
    );

    if res == u32::MAX {
        log::error!("Failed to get raw input data");
        return;
    }

    let raw: &RAWINPUT = &*(buffer.as_ptr() as *const RAWINPUT);

    if raw.header.dwType == RIM_TYPEHID {
        let hid = raw.data.hid;
        let report_size = hid.dwSizeHid as usize;
        let count = hid.dwCount as usize;
        let data_ptr = hid.bRawData.as_ptr();

        for i in 0..count {
            let report = std::slice::from_raw_parts(
                data_ptr.add(i * report_size),
                report_size,
            );

            let events = hid_parser::parse_a1314_hid_report(report);

            GLOBAL_MAPPER.with(|gm| {
                if let Some(mapper_rc) = &*gm.borrow() {
                    let mut mapper = mapper_rc.borrow_mut();
                    for (usage_page, usage, value) in events {
                        mapper.handle_hid_event(usage_page, usage, value);
                    }
                }
            });
        }
    }
}

unsafe extern "system" fn keyboard_hook_proc(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode >= 0 {
        let kbd = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        
        // Skip inputs injected by this daemon to prevent feedback loops
        if kbd.dwExtraInfo == action_executor::DAEMON_INJECTION_TAG as usize {
            return CallNextHookEx(None, ncode, wparam, lparam);
        }

        let msg = wparam.0 as u32;
        let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
        let vk = kbd.vkCode;
        
        // Translate VK to HID Usage (Usage Page 0x07)
        let usage = match vk {
            0x41..=0x5A => vk as u16 - 0x41 + 4, // A-Z (0x41='A' -> Usage 0x04)
            0x30 => 0x27, // '0' -> Usage 0x27
            0x31..=0x39 => vk as u16 - 0x31 + 0x1E, // 1-9 (0x31='1' -> Usage 0x1E)
            0x0D => 0x28, // ENTER -> Usage 0x28
            0x1B => 0x29, // ESCAPE -> Usage 0x29
            0x08 => 0x2A, // BACKSPACE -> Usage 0x2A
            0x09 => 0x2B, // TAB -> Usage 0x2B
            0x20 => 0x2C, // SPACE -> Usage 0x2C
            0x25 => 0x50, // LEFT -> Usage 0x50
            0x26 => 0x52, // UP -> Usage 0x52
            0x27 => 0x4F, // RIGHT -> Usage 0x4F
            0x28 => 0x51, // DOWN -> Usage 0x51
            0x2E => 0x4C, // DELETE -> Usage 0x4C (Forward Delete)
            0x70..=0x7B => vk as u16 - 0x70 + 0x3A, // F1-F12 (0x70=F1 -> Usage 0x3A)
            _ => 0,
        };

        if usage != 0 {
            let mut should_suppress = false;
            GLOBAL_MAPPER.with(|gm| {
                if let Some(mapper_rc) = &*gm.borrow() {
                    let mut mapper = mapper_rc.borrow_mut();
                    
                    if !is_up {
                        // Check for mapping and trigger it
                        if mapper.try_trigger_mapping(0x07, usage, 1) {
                            SUPPRESSED_KEYS.with(|sk| sk.borrow_mut().insert(vk));
                            should_suppress = true;
                        }
                    } else {
                        // If it's an UP event, check if we suppressed the corresponding DOWN
                        let was_suppressed = SUPPRESSED_KEYS.with(|sk| sk.borrow_mut().remove(&vk));
                        if was_suppressed {
                            should_suppress = true;
                        }
                        // Always update state for modifiers etc.
                        mapper.handle_hid_event(0x07, usage, 0);
                    }
                }
            });

            if should_suppress {
                return LRESULT(1); // Suppress the physical key event
            }
        }
    }
    CallNextHookEx(None, ncode, wparam, lparam)
}

fn install_service() -> windows::core::Result<()> {
    use windows::Win32::System::Registry::*;
    use windows::core::HSTRING;

    log::info!("Installing A1314 Daemon to start with Windows...");

    let exe_path = std::env::current_exe()
        .expect("Failed to get executable path");
    
    let key_path = HSTRING::from("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let value_name = HSTRING::from("A1314Daemon");

    unsafe {
        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            &key_path,
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );

        if result.is_err() {
            log::error!("Failed to open registry key: {:?}", result);
            println!("Failed to install. Run as administrator if needed.");
            return result.ok();
        }

        let exe_path_str = exe_path.to_string_lossy();
        let exe_path_wide: Vec<u16> = exe_path_str.encode_utf16().chain(std::iter::once(0)).collect();

        let result = RegSetValueExW(
            hkey,
            &value_name,
            0,
            REG_SZ,
            Some(&exe_path_wide.iter().flat_map(|&c| c.to_le_bytes()).collect::<Vec<u8>>()),
        );

        let _ = RegCloseKey(hkey);

        if result.is_ok() {
            log::info!("Successfully installed A1314 Daemon to start with Windows");
            println!("âœ“ A1314 Daemon installed successfully!");
            println!("  The daemon will now start automatically when you log in.");
            println!("  To uninstall, run: {} --uninstall", exe_path.file_name().unwrap().to_string_lossy());
        } else {
            log::error!("Failed to set registry value: {:?}", result);
            println!("Failed to install. Run as administrator if needed.");
        }

        result.ok()
    }
}

fn uninstall_service() -> windows::core::Result<()> {
    use windows::Win32::System::Registry::*;
    use windows::core::HSTRING;

    log::info!("Uninstalling A1314 Daemon from Windows startup...");

    let key_path = HSTRING::from("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let value_name = HSTRING::from("A1314Daemon");

    unsafe {
        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            &key_path,
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );

        if result.is_err() {
            log::error!("Failed to open registry key: {:?}", result);
            println!("Failed to uninstall. The daemon may not be installed.");
            return result.ok();
        }

        let result = RegDeleteValueW(hkey, &value_name);
        let _ = RegCloseKey(hkey);

        if result.is_ok() {
            log::info!("Successfully uninstalled A1314 Daemon from Windows startup");
            println!("âœ“ A1314 Daemon uninstalled successfully!");
            println!("  The daemon will no longer start automatically.");
        } else {
            log::error!("Failed to delete registry value: {:?}", result);
            println!("Failed to uninstall. The daemon may not be installed.");
        }

        result.ok()
    }
}

fn print_help() {
    println!("{} v{} - Apple Wireless Keyboard Mapper for Windows", 
             env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!();
    println!("USAGE:");
    println!("  a1314_daemon.exe [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  --install      Install daemon to start with Windows");
    println!("  --uninstall    Remove daemon from Windows startup");
    println!("  --help, -h     Show this help message");
    println!();
    println!("NORMAL OPERATION:");
    println!("  Run without arguments to start the daemon.");
    println!("  Use the system tray icon to:");
    println!("    â€¢ Reload configuration");
    println!("    â€¢ Reset to default configuration");
    println!("    â€¢ Exit the daemon");
    println!();
    println!("CONFIGURATION:");
    println!("  Edit A1314_mapping.txt in the same directory as the executable.");
    println!("  Changes are automatically reloaded when you save the file.");
}

fn widestring(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}