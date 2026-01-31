// --- src/hid_parser.rs ---
use std::collections::HashSet;
use std::sync::Mutex;

// Global state to track previously pressed keys for detecting releases
static PREVIOUS_KEYS: Mutex<Option<HashSet<(u16, u16)>>> = Mutex::new(None);

/// Parses Apple A1314 HID reports and extracts usage page, usage, and value tuples
/// Returns key-down (value=1) and key-up (value=0) events.
pub fn parse_a1314_hid_report(report: &[u8]) -> Vec<(u16, u16, i32)> {
    let mut events = Vec::new();

    if report.len() < 2 {
        return events;
    }

    // Debug: log raw report (uncomment for troubleshooting)
    // eprintln!("HID Report: {:02X?}", report);

    let report_id = report[0];
    let mut current_stateful_keys = HashSet::new(); // Keys that maintain a "pressed" state

    // --- Process Report based on Report ID ---
    match report_id {
        // Standard keyboard report (0x01)
        0x01 => {
            if report.len() >= 8 {
                // Modifiers in byte 1 (Usage Page 0x07)
                let modifiers = report[1];
                let modifier_codes = [
                    0xE0, // LEFT_CTRL
                    0xE1, // LEFT_SHIFT
                    0xE2, // LEFT_ALT
                    0xE3, // LEFT_GUI
                    0xE4, // RIGHT_CTRL
                    0xE5, // RIGHT_SHIFT
                    0xE6, // RIGHT_ALT
                    0xE7, // RIGHT_GUI
                ];

                for (bit, code) in modifier_codes.iter().enumerate() {
                    let key_tuple = (0x07, *code);
                    if modifiers & (1 << bit) != 0 {
                        current_stateful_keys.insert(key_tuple);
                    }
                }

                // Key codes in bytes 3-8 (Usage Page 0x07)
                for i in 3..8 {
                    if report[i] != 0 && report[i] != 1 { // 0 = no key, 1 = error rollover
                        let key_tuple = (0x07, report[i] as u16);
                        current_stateful_keys.insert(key_tuple);
                    }
                }
            }
        }
        
        // Consumer control report (0x02 or 0x03) (Usage Page 0x0C)
        // Now adding these to stateful keys if they represent a toggle/hold.
        // EJECT (0C:00B8) is handled here.
        0x02 | 0x03 => {
            if report.len() >= 3 {
                let usage = u16::from_le_bytes([report[1], report[2]]);
                if usage != 0 {
                    let key_tuple = (0x0C, usage);
                    // Add consumer control keys to stateful tracking,
                    // so we can detect their press and release like other keys.
                    current_stateful_keys.insert(key_tuple);
                }
            }
        }
        
        // Apple vendor-specific (Fn key state) (Usage Page 0xFF00)
        0x05 => {
            if report.len() >= 2 {
                // Fn key state is typically in byte 1, bit 0
                let fn_state = (report[1] & 0x01) != 0;
                let key_tuple = (0xFF00, 0x0003); // Specific Fn state usage
                if fn_state {
                    current_stateful_keys.insert(key_tuple);
                }
            }
        }
        
        _ => {
            // Generic fallback for unknown report types - treated as momentary
            if report.len() >= 4 {
                let usage_page = u16::from_le_bytes([report[1], report[2]]);
                let usage = report[3] as u16;
                if usage != 0 {
                    // Generic events are also treated as momentary
                    events.push((usage_page, usage, 1));
                }
            }
        }
    }

    // --- Compare Stateful Keys with Previous State to Detect Releases ---
    let mut prev_state_lock = PREVIOUS_KEYS.lock().unwrap();
    
    if let Some(ref previous_stateful_keys) = *prev_state_lock {
        // Key-up events for stateful keys: keys that were pressed before but aren't now
        for key in previous_stateful_keys.iter() {
            if !current_stateful_keys.contains(key) {
                events.push((key.0, key.1, 0));
            }
        }
        
        // Key-down events for stateful keys: keys that are pressed now but weren't before
        for key in current_stateful_keys.iter() {
            if !previous_stateful_keys.contains(key) {
                events.push((key.0, key.1, 1));
            }
        }
    } else {
        // First time initialization: all currently pressed stateful keys are new key-down events
        for key in current_stateful_keys.iter() {
            events.push((key.0, key.1, 1));
        }
    }

    // Update previous state for stateful keys
    *prev_state_lock = Some(current_stateful_keys);

    events
}