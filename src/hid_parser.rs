// --- src/hid_parser.rs ---
use std::collections::HashSet;
use std::sync::Mutex;

// Constants for HID report values
const NO_KEY: u8 = 0;
const ERROR_ROLLOVER: u8 = 1;

// Global state to track previously pressed keys for detecting releases
static PREVIOUS_KEYS: Mutex<Option<HashSet<(u16, u16)>>> = Mutex::new(None);

/// Parses Apple A1314 HID reports and extracts usage page, usage, and value tuples
/// Returns key-down (value=1) and key-up (value=0) events.
pub fn parse_a1314_hid_report(report: &[u8]) -> Vec<(u16, u16, i32)> {
    let mut events = Vec::new();

    if report.len() < 2 {
        log::warn!("HID report too short: {} bytes (expected at least 2)", report.len());
        return events;
    }

    // Debug: log raw report (controlled by log level)
    log::debug!("HID Report (ID={:02X}, len={}): {:02X?}", report[0], report.len(), report);

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

                // Key codes in bytes 3 onwards (Usage Page 0x07)
                // Standard 6-key rollover reports are 8 bytes total
                for i in 3..report.len() {
                    if report[i] != NO_KEY && report[i] != ERROR_ROLLOVER {
                        let key_tuple = (0x07, report[i] as u16);
                        current_stateful_keys.insert(key_tuple);
                    }
                }
            } else {
                log::warn!("Standard keyboard report too short: {} bytes (expected 8)", report.len());
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
            } else {
                log::warn!("Consumer control report too short: {} bytes (expected 3)", report.len());
            }
        }
        
        // Apple vendor-specific (Fn key state) (Usage Page 0xFF00)
        // Report 0x05 (typically USB) or 0x11 (typically Bluetooth)
        0x05 | 0x11 => {
            if report.len() >= 2 {
                // Heuristic: check bit 0 (0x01) for report 0x05, 
                // and bit 4 (0x10) for report 0x11 as discovered in logs.
                let mut fn_state = false;
                if report_id == 0x05 {
                    fn_state = (report[1] & 0x01) != 0;
                } else if report_id == 0x11 {
                    fn_state = (report[1] & 0x10) != 0;
                    
                    // Also check for Eject bit (0x08) in Bluetooth report 0x11
                    let eject_state = (report[1] & 0x08) != 0;
                    if eject_state {
                        current_stateful_keys.insert((0x0C, 0x00B8)); // Standard Eject usage
                    }
                }

                let key_tuple = (0xFF00, 0x0003); // Specific Fn state usage
                if fn_state {
                    current_stateful_keys.insert(key_tuple);
                }
            } else {
                log::warn!("Vendor-specific report too short: {} bytes", report.len());
            }
        }
        
        _ => {
            // Generic fallback for unknown report types - treated as momentary
            log::debug!("Unknown HID report ID: 0x{:02X}", report_id);
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
    // Handle lock poisoning by recovering the inner data
    let mut prev_state_lock = PREVIOUS_KEYS.lock().unwrap_or_else(|poisoned| {
        log::error!("PREVIOUS_KEYS mutex was poisoned, recovering...");
        poisoned.into_inner()
    });

    // Modifier state is reported in the same report as the key it modifies, so the events must
    // not come out in hash order: Fn+Shift+Backspace has to arrive as Shift-down, Fn-down,
    // Backspace-down. Otherwise the key is sometimes resolved before its modifiers are known, and
    // the same keypress resolves to different mappings from one repeat to the next.
    let mut state_ups: Vec<(u16, u16, i32)> = Vec::new();
    let mut state_downs: Vec<(u16, u16, i32)> = Vec::new();
    let mut key_ups: Vec<(u16, u16, i32)> = Vec::new();
    let mut key_downs: Vec<(u16, u16, i32)> = Vec::new();

    if let Some(ref previous_stateful_keys) = *prev_state_lock {
        // Key-up events for stateful keys: keys that were pressed before but aren't now
        for key in previous_stateful_keys.iter() {
            if !current_stateful_keys.contains(key) {
                let event = (key.0, key.1, 0);
                if is_modifier_state(*key) {
                    state_ups.push(event);
                } else {
                    key_ups.push(event);
                }
            }
        }

        // Key-down events for stateful keys: keys that are pressed now but weren't before
        for key in current_stateful_keys.iter() {
            if !previous_stateful_keys.contains(key) {
                log::debug!("Key-Down: {:04X}:{:04X}", key.0, key.1);
                let event = (key.0, key.1, 1);
                if is_modifier_state(*key) {
                    state_downs.push(event);
                } else {
                    key_downs.push(event);
                }
            }
        }
    } else {
        // First time initialization: all currently pressed stateful keys are new key-down events
        for key in current_stateful_keys.iter() {
            let event = (key.0, key.1, 1);
            if is_modifier_state(*key) {
                state_downs.push(event);
            } else {
                key_downs.push(event);
            }
        }
    }

    for bucket in [&mut state_ups, &mut state_downs, &mut key_ups, &mut key_downs] {
        bucket.sort_unstable();
    }

    let mut events = Vec::with_capacity(
        state_ups.len() + state_downs.len() + key_ups.len() + key_downs.len(),
    );
    events.extend(state_ups);
    events.extend(state_downs);
    events.extend(key_ups);
    events.extend(key_downs);

    // Update previous state for stateful keys
    *prev_state_lock = Some(current_stateful_keys);

    events
}

/// A key whose only job is to change the modifier state: Shift/Ctrl/Alt/GUI, Apple's Fn state,
/// and Eject. These are applied before any ordinary key from the same report.
fn is_modifier_state(key: (u16, u16)) -> bool {
    match key.0 {
        0x07 => (0xE0..=0xE7).contains(&key.1),
        0x0C => key.1 == 0x00B8,      // Eject
        0xFF00 => key.1 == 0x0003,    // Fn state
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One standard keyboard report: report id, modifier byte, reserved byte, then up to six usages.
    fn standard_report(modifiers: u8, keys: &[u8]) -> Vec<u8> {
        let mut report = vec![0x01, modifiers, 0x00];
        report.extend_from_slice(keys);
        report.resize(8, 0);
        report
    }

    /// The report state is process-global, so both checks live in one test to keep them from
    /// interfering with each other.
    #[test]
    fn modifier_state_comes_first_and_is_not_re_emitted() {
        *PREVIOUS_KEYS.lock().unwrap() = None;

        // Shift (modifier bit 1) and Backspace in the same report, which is what Fn+Shift+
        // Backspace produces once Fn is already held.
        let report = standard_report(0x02, &[0x2A]);
        let events = parse_a1314_hid_report(&report);

        let shift = events
            .iter()
            .position(|event| *event == (0x07, 0xE1, 1))
            .expect("shift key-down");
        let backspace = events
            .iter()
            .position(|event| *event == (0x07, 0x2A, 1))
            .expect("backspace key-down");
        assert!(
            shift < backspace,
            "the modifier has to be applied before the key it modifies: {events:?}"
        );

        // An identical report repeats nothing: only changes are events.
        let again = parse_a1314_hid_report(&report);
        assert!(again.is_empty(), "an unchanged report produced events: {again:?}");

        // Releasing the key emits only its key-up, and releasing Shift emits only Shift-up.
        let released = parse_a1314_hid_report(&standard_report(0x00, &[]));
        assert_eq!(released, vec![(0x07, 0xE1, 0), (0x07, 0x2A, 0)]);

        *PREVIOUS_KEYS.lock().unwrap() = None;
    }
}
