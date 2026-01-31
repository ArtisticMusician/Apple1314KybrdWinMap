// --- START OF FILE src/key_mapper.rs ---
mod variable_maps; // Import the new module

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::action_executor::{Action, execute_action};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HidKey {
    pub usage_page: u16,
    pub usage: u16,
}

#[derive(Default)]
struct KeyMaps {
    normal: HashMap<HidKey, Action>,
    fn_map: HashMap<HidKey, Action>,
    shift_map: HashMap<HidKey, Action>,      // Map for SHIFT as modifier
    eject_map: HashMap<HidKey, Action>,      // Map for EJECT as modifier
    eject_fn_map: HashMap<HidKey, Action>,   // Map for EJECT+FN as modifier
}

pub struct KeyMapper {
    maps: KeyMaps,
    fn_down: bool,
    shift_down: bool,    // Field to track SHIFT state (either left or right)
    eject_down: bool,    // Field to track EJECT state
}

// Define the HID key for EJECT (from variable_maps)
const EJECT_HID_KEY: HidKey = HidKey { usage_page: 0x0C, usage: 0x00B8 };

// Define the HID key for FN_STATE (from variable_maps)
const FN_STATE_HID_KEY: HidKey = HidKey { usage_page: 0xFF00, usage: 0x0003 };

// Define the HID keys for SHIFT (from variable_maps)
const LEFT_SHIFT_HID_KEY: HidKey = HidKey { usage_page: 0x07, usage: 0x00E1 };
const RIGHT_SHIFT_HID_KEY: HidKey = HidKey { usage_page: 0x07, usage: 0x00E5 };

impl KeyMapper {
    pub fn new() -> Self {
        Self {
            maps: KeyMaps::default(),
            fn_down: false,
            shift_down: false,
            eject_down: false,
        }
    }

    pub fn load_mapping_file<P: AsRef<Path>>(&mut self, path: P) {
        let path_ref = path.as_ref();
        let text = match fs::read_to_string(path_ref) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to read mapping file '{}': {}", path_ref.display(), e);
                return;
            }
        };

        log::info!("Loading mappings from: {}", path_ref.display());

        let mut normal = HashMap::new();
        let mut fn_map = HashMap::new();
        let mut shift_map = HashMap::new();
        let mut eject_map = HashMap::new();
        let mut eject_fn_map = HashMap::new();

        let mut line_count = 0;
        let mut error_count = 0;

        for (line_no, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            line_count += 1;

            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() != 2 {
                log::error!("Invalid mapping syntax at line {}: {}", line_no + 1, line);
                log::info!("  Expected format: KEY = ACTION");
                error_count += 1;
                continue;
            }

            let lhs_str = parts[0];
            let rhs_str = parts[1].to_string(); // Keep as String for Action parsing

            // Check for SHIFT+ prefix first (can be LEFT_SHIFT+ or RIGHT_SHIFT+)
            let (is_shift, rest_after_shift) = if let Some(rest) = lhs_str.strip_prefix("LEFT_SHIFT+") {
                (true, rest.trim())
            } else if let Some(rest) = lhs_str.strip_prefix("RIGHT_SHIFT+") {
                (true, rest.trim())
            } else {
                (false, lhs_str)
            };

            let (is_eject, rest_after_eject) = if let Some(rest) = rest_after_shift.strip_prefix("EJECT+") {
                (true, rest.trim())
            } else {
                (false, rest_after_shift)
            };

            let (is_fn, key_name) = if let Some(rest) = rest_after_eject.strip_prefix("FN+") {
                (true, rest.trim())
            } else {
                (false, rest_after_eject)
            };

            // Lookup the HidKey from the hardcoded map
            let hid_key = match variable_maps::STRING_TO_HID_KEY.get(key_name) {
                Some(key) => *key,
                None => {
                    log::error!("Unknown key name at line {}: '{}'", line_no + 1, key_name);
                    log::info!("  Check src/variable_maps.rs for valid key names");
                    error_count += 1;
                    continue;
                }
            };

            // Parse the Action for the RHS
            let action = if let Some(rest) = rhs_str.strip_prefix("RUN(\"") {
                if let Some(end) = rest.rfind("\")") {
                    let path = &rest[..end];
                    Action::Run(path.to_string())
                } else {
                    log::error!("Malformed RUN() syntax at line {}: '{}'", line_no + 1, rhs_str);
                    log::info!("  Expected format: RUN(\"path/to/program.exe\")");
                    error_count += 1;
                    Action::KeyCombo(rhs_str) // Fallback
                }
            } else if let Some(rest) = rhs_str.strip_prefix("APPCOMMAND(") {
                if let Some(end) = rest.find(')') {
                    let cmd_str = &rest[..end];
                    if let Ok(cmd_val) = cmd_str.parse::<u32>() {
                        Action::AppCommand(cmd_val)
                    } else {
                        log::error!("Invalid APPCOMMAND value at line {}: '{}'", line_no + 1, rhs_str);
                        log::info!("  Expected a number, e.g., APPCOMMAND(46)");
                        error_count += 1;
                        Action::KeyCombo(rhs_str) // Fallback
                    }
                } else {
                    log::error!("Malformed APPCOMMAND syntax at line {}: '{}'", line_no + 1, rhs_str);
                    log::info!("  Expected format: APPCOMMAND(number)");
                    error_count += 1;
                    Action::KeyCombo(rhs_str) // Fallback
                }
            }
            else {
                // For direct string actions like "MUTE", "WIN+TAB", look them up
                match variable_maps::STRING_TO_ACTION.get(rhs_str.as_str()) {
                    Some(action) => action.clone(),
                    None => {
                        // Fallback to KeyCombo if not a recognized explicit action
                        Action::KeyCombo(rhs_str) 
                    }
                }
            };

            if is_eject && is_fn {
                eject_fn_map.insert(hid_key, action);
            } else if is_eject {
                eject_map.insert(hid_key, action);
            } else if is_shift {
                shift_map.insert(hid_key, action);
            } else if is_fn {
                fn_map.insert(hid_key, action);
            } else {
                normal.insert(hid_key, action);
            }
        }

        self.maps = KeyMaps { normal, fn_map, shift_map, eject_map, eject_fn_map };
        
        log::info!("Loaded {} mappings from {} lines", 
                   self.maps.normal.len() + self.maps.fn_map.len() + 
                   self.maps.shift_map.len() + self.maps.eject_map.len() + 
                   self.maps.eject_fn_map.len(),
                   line_count);
        log::info!("  Normal: {}, Fn: {}, Shift: {}, Eject: {}, Eject+Fn: {}", 
                   self.maps.normal.len(), 
                   self.maps.fn_map.len(), 
                   self.maps.shift_map.len(),
                   self.maps.eject_map.len(), 
                   self.maps.eject_fn_map.len());
        
        if error_count > 0 {
            log::warn!("{} errors encountered while loading mappings", error_count);
        }
        
        if self.maps.normal.is_empty() && self.maps.fn_map.is_empty() && 
           self.maps.shift_map.is_empty() && self.maps.eject_map.is_empty() && 
           self.maps.eject_fn_map.is_empty() {
            log::warn!("No valid mappings loaded! Check your mapping file syntax");
        }
    }

    pub fn handle_hid_event(&mut self, usage_page: u16, usage: u16, value: i32) {
        let key = HidKey { usage_page, usage };

        // Update Fn state
        if key == FN_STATE_HID_KEY {
            self.fn_down = value != 0;
            log::trace!("Fn key: {}", if self.fn_down { "DOWN" } else { "UP" });
            return;
        }

        // Update SHIFT state (either left or right)
        if key == LEFT_SHIFT_HID_KEY || key == RIGHT_SHIFT_HID_KEY {
            self.shift_down = value != 0;
            log::trace!("Shift key: {}", if self.shift_down { "DOWN" } else { "UP" });
            return;
        }

        // Update EJECT state
        if key == EJECT_HID_KEY {
            self.eject_down = value != 0;
            log::trace!("Eject key: {}", if self.eject_down { "DOWN" } else { "UP" });
            return;
        }

        // Only act on key-down for triggering actions
        if value == 0 {
            return;
        }

        // Determine which map to use based on modifier states
        // Priority: EJECT+FN > EJECT > SHIFT > FN > NORMAL
        let action = if self.eject_down && self.fn_down {
            self.maps.eject_fn_map.get(&key)
        } else if self.eject_down {
            self.maps.eject_map.get(&key)
        } else if self.shift_down {
            self.maps.shift_map.get(&key)
        } else if self.fn_down {
            self.maps.fn_map.get(&key)
        } else {
            self.maps.normal.get(&key)
        };

        if let Some(action) = action {
            log::debug!("Executing action for key {:04X}:{:04X} (modifiers: Fn={}, Shift={}, Eject={}): {:?}",
                       usage_page, usage, self.fn_down, self.shift_down, self.eject_down, action);
            execute_action(action);
        }
    }
}
