// --- START OF FILE src/main.rs ---
mod hid_parser;
mod key_mapper;
mod action_executor;
mod variable_maps;

use std::cell::RefCell;
use std::rc::Rc;
use std::ptr::null_mut;
use std::ffi::c_void;

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
};

use key_mapper::KeyMapper;

// Thread-local storage for the key mapper
// IMPORTANT: This assumes all HID input processing happens on the window message thread.
// The Windows raw input API guarantees WM_INPUT messages are delivered to the thread
// that created the window, so this assumption holds as long as we don't spawn additional
// threads to process HID input.
thread_local! {
    static GLOBAL_MAPPER: RefCell<Option<Rc<RefCell<KeyMapper>>>> = RefCell::new(None);
}

fn main() -> windows::core::Result<()> {
    // Initialize logging
    // Set RUST_LOG environment variable to control log level
    // Example: RUST_LOG=debug for verbose output, RUST_LOG=info for normal output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .init();

    log::info!("A1314 Daemon starting...");
    log::info!("Log level: {} (set RUST_LOG environment variable to change)", log::max_level());

    // Force initialization of lazy_static maps
    let _ = variable_maps::STRING_TO_HID_KEY.len();
    let _ = variable_maps::STRING_TO_ACTION.len();

    // Simplified mapping file path - always look next to the executable
    let exe_path = std::env::current_exe()
        .expect("Failed to get executable path");
    let exe_dir = exe_path.parent()
        .expect("Failed to get executable directory");
    let mapping_path = exe_dir.join("A1314_mapping.txt");

    log::info!("Executable location: {}", exe_path.display());
    log::info!("Looking for mapping file: {}", mapping_path.display());

    if !mapping_path.exists() {
        log::error!("Mapping file not found!");
        log::error!("Expected location: {}", mapping_path.display());
        log::error!("Please ensure A1314_mapping.txt is in the same directory as the executable");
        std::process::exit(1);
    }

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

        register_raw_input(hwnd)?;
        log::info!("Raw input registered successfully");
        log::info!("Daemon is now running. Press Ctrl+C to exit.");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(null_mut()), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    log::info!("Daemon shutting down");
    Ok(())
}

unsafe fn register_raw_input(hwnd: HWND) -> windows::core::Result<()> {
    let devices = [
        RAWINPUTDEVICE {
            usUsagePage: 0x01, // Generic Desktop Controls
            usUsage: 0x06,     // Keyboard
            dwFlags: RAWINPUTDEVICE_FLAGS(RIDEV_INPUTSINK.0),
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: 0x0C, // Consumer (media keys like EJECT)
            usUsage: 0x01,     // Consumer Control - top-level collection, registers all 0x0C usages
            dwFlags: RAWINPUTDEVICE_FLAGS(RIDEV_INPUTSINK.0),
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: 0xFF00, // Apple Vendor-Specific Usage Page
            usUsage: 0x01,       // Likely for the Fn key state (top-level collection)
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
            WM_DESTROY => {
                log::info!("Received WM_DESTROY, shutting down");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

// HID Raw Input Type constant
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

fn widestring(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
