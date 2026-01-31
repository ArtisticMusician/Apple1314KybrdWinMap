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
    eject_map: HashMap<HidKey, Action>,      // Map for EJECT as modifier
    eject_fn_map: HashMap<HidKey, Action>,   // Map for EJECT+FN as modifier
}

pub struct KeyMapper {
    maps: KeyMaps,
    fn_down: bool,
    eject_down: bool, // Field to track EJECT state
}

// Define the HID key for EJECT (from variable_maps)
const EJECT_HID_KEY: HidKey = HidKey { usage_page: 0x0C, usage: 0x00B8 };

// Define the HID key for FN_STATE (from variable_maps)
const FN_STATE_HID_KEY: HidKey = HidKey { usage_page: 0xFF00, usage: 0x0003 };

impl KeyMapper {
    pub fn new() -> Self {
        Self {
            maps: KeyMaps::default(),
            fn_down: false,
            eject_down: false,
        }
    }

    pub fn load_mapping_file<P: AsRef<Path>>(&mut self, path: P) {
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to read mapping file: {e}");
                return;
            }
        };

        let mut normal = HashMap::new();
        let mut fn_map = HashMap::new();
        let mut eject_map = HashMap::new();
        let mut eject_fn_map = HashMap::new();

        // No need for a first pass for variables anymore
        for (line_no, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() != 2 {
                eprintln!("Invalid mapping line {}: {}", line_no + 1, line);
                continue;
            }

            let lhs_str = parts[0];
            let rhs_str = parts[1].to_string(); // Keep as String for Action parsing

            let (is_eject, rest_after_eject) = if let Some(rest) = lhs_str.strip_prefix("EJECT+") {
                (true, rest.trim())
            } else {
                (false, lhs_str)
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
                    eprintln!("Unknown key name on LHS at line {}: {}", line_no + 1, key_name);
                    continue;
                }
            };

            // Parse the Action for the RHS
            let action = if let Some(rest) = rhs_str.strip_prefix("RUN(\"") {
                if let Some(end) = rest.rfind("\")") {
                    let path = &rest[..end];
                    Action::Run(path.to_string())
                } else {
                    Action::KeyCombo(rhs_str) // Fallback if RUN syntax is malformed
                }
            } else if let Some(rest) = rhs_str.strip_prefix("APPCOMMAND(") {
                if let Some(end) = rest.find(')') {
                    let cmd_str = &rest[..end];
                    if let Ok(cmd_val) = cmd_str.parse::<u32>() {
                        Action::AppCommand(cmd_val)
                    } else {
                        eprintln!("Invalid APPCOMMAND value at line {}: {}", line_no + 1, rhs_str);
                        Action::KeyCombo(rhs_str) // Fallback if APPCOMMAND value is invalid
                    }
                } else {
                    eprintln!("Malformed APPCOMMAND at line {}: {}", line_no + 1, rhs_str);
                    Action::KeyCombo(rhs_str) // Fallback if APPCOMMAND syntax is malformed
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
            } else if is_fn {
                fn_map.insert(hid_key, action);
            } else {
                normal.insert(hid_key, action);
            }
        }

        self.maps = KeyMaps { normal, fn_map, eject_map, eject_fn_map };
        println!("Loaded {} normal, {} Fn, {} Eject, {} Eject+Fn mappings", 
                 self.maps.normal.len(), 
                 self.maps.fn_map.len(), 
                 self.maps.eject_map.len(), 
                 self.maps.eject_fn_map.len());
    }

    pub fn handle_hid_event(&mut self, usage_page: u16, usage: u16, value: i32) {
        let key = HidKey { usage_page, usage };

        // Update Fn state
        if key == FN_STATE_HID_KEY {
            self.fn_down = value != 0;
            return;
        }

        // Update EJECT state
        if key == EJECT_HID_KEY {
            self.eject_down = value != 0;
            return;
        }

        // Only act on key-down for triggering actions
        if value == 0 {
            return;
        }

        // Determine which map to use based on modifier states
        let action = if self.eject_down && self.fn_down {
            self.maps.eject_fn_map.get(&key)
        } else if self.eject_down {
            self.maps.eject_map.get(&key)
        } else if self.fn_down {
            self.maps.fn_map.get(&key)
        } else {
            self.maps.normal.get(&key)
        };

        if let Some(action) = action {
            execute_action(action);
        }
    }
}