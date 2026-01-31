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
}

fn main() -> windows::core::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
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

    log::info!("A1314 Daemon v0.2.0 starting...");
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
            PCWSTR(widestring("A1314Daemon").as_ptr()),
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
        let hwnd_for_thread = hwnd;
        std::thread::spawn(move || {
            handle_file_watch_events(rx, hwnd_for_thread);
        });

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(null_mut()), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Keep watcher alive until shutdown
        drop(watcher);
    }

    log::info!("Daemon shutting down");
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

fn create_system_tray(exe_dir: &std::path::Path) -> Result<(), String> {
    // Load icon from file
    let icon_path = exe_dir.join("RottenApple_1.ico");
    let icon = if icon_path.exists() {
        Icon::from_path(&icon_path, Some((32, 32)))
            .map_err(|e| format!("Failed to load icon: {}", e))?
    } else {
        log::warn!("Icon file not found at {}, using default", icon_path.display());
        Icon::from_rgba(vec![255; 32 * 32 * 4], 32, 32)
            .map_err(|e| format!("Failed to create default icon: {}", e))?
    };

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

    // Handle menu events
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = tray_icon::menu::MenuEvent::receiver().recv() {
                MAIN_WINDOW.with(|wnd| {
                    if let Some(hwnd) = *wnd.borrow() {
                        unsafe {
                            if event.id == reload_item.id() {
                                let _ = PostMessageW(hwnd, WM_RELOAD_CONFIG, WPARAM(0), LPARAM(0));
                            } else if event.id == reset_item.id() {
                                let _ = PostMessageW(hwnd, WM_RESET_CONFIG, WPARAM(0), LPARAM(0));
                            } else if event.id == exit_item.id() {
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

unsafe fn handle_raw_input(lparam: LPARAM) {
    let hrawinput = HRAWINPUT(lparam.0 as *mut c_void);
    
    let mut header = RAWINPUTHEADER::default();
    let mut header_size = std::mem::size_of::<RAWINPUTHEADER>() as u32;
    let res = GetRawInputData(
        hrawinput,
        RID_INPUT,
        Some(&mut header as *mut _ as *mut c_void),
        &mut header_size,
        std::mem::size_of::<RAWINPUTHEADER>() as u32,
    );

    if res == u32::MAX {
        log::error!("Failed to get raw input header");
        return;
    }

    let mut buffer = vec![0u8; header.dwSize as usize];
    let mut size = header.dwSize;
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
                    for (usage_page, usage, value) in events {
                        mapper_rc
                            .borrow_mut()
                            .handle_hid_event(usage_page, usage, value);
                    }
                }
            });
        }
    }
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
            return result;
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

        RegCloseKey(hkey);

        if result.is_ok() {
            log::info!("Successfully installed A1314 Daemon to start with Windows");
            println!("✓ A1314 Daemon installed successfully!");
            println!("  The daemon will now start automatically when you log in.");
            println!("  To uninstall, run: {} --uninstall", exe_path.file_name().unwrap().to_string_lossy());
        } else {
            log::error!("Failed to set registry value: {:?}", result);
            println!("Failed to install. Run as administrator if needed.");
        }

        result
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
            return result;
        }

        let result = RegDeleteValueW(hkey, &value_name);
        RegCloseKey(hkey);

        if result.is_ok() {
            log::info!("Successfully uninstalled A1314 Daemon from Windows startup");
            println!("✓ A1314 Daemon uninstalled successfully!");
            println!("  The daemon will no longer start automatically.");
        } else {
            log::error!("Failed to delete registry value: {:?}", result);
            println!("Failed to uninstall. The daemon may not be installed.");
        }

        result
    }
}

fn print_help() {
    println!("A1314 Daemon v0.2.0 - Apple Wireless Keyboard Mapper for Windows");
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
    println!("    • Reload configuration");
    println!("    • Reset to default configuration");
    println!("    • Exit the daemon");
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
