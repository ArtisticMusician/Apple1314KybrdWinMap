// --- START OF FILE src/key_mapper.rs ---
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::action_executor::{Action, execute_action};
use crate::variable_maps::{STRING_TO_HID_KEY, STRING_TO_ACTION};

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
            let hid_key = match STRING_TO_HID_KEY.get(key_name) {
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
                match STRING_TO_ACTION.get(rhs_str.as_str()) {
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

    /// The one place that decides which mapping a key resolves to, given the modifier state.
    ///
    /// Specific first, then the less specific reading of the same keypress: a key pressed with
    /// Fn *and* Shift resolves through the Shift map, but falls back to the Fn map rather than
    /// dropping to no mapping at all. That fallback is the difference between "Fn+Shift+Backspace
    /// does what Fn+Backspace does, with Shift held" and "the key does nothing and the physical
    /// key leaks through as a plain Backspace".
    pub fn resolve(&self, key: HidKey) -> Option<&Action> {
        if self.eject_down && self.fn_down {
            if let Some(action) = self.maps.eject_fn_map.get(&key) {
                return Some(action);
            }
        }
        if self.eject_down {
            if let Some(action) = self.maps.eject_map.get(&key) {
                return Some(action);
            }
        }
        if self.shift_down {
            if let Some(action) = self.maps.shift_map.get(&key) {
                return Some(action);
            }
        }
        if self.fn_down {
            if let Some(action) = self.maps.fn_map.get(&key) {
                return Some(action);
            }
        }
        if self.shift_down {
            if let Some(action) = self.maps.shift_map.get(&key) {
                return Some(action);
            }
        }
        self.maps.normal.get(&key)
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

        if let Some(action) = self.resolve(key) {
            log::debug!("Executing action for key {:04X}:{:04X} (modifiers: Fn={}, Shift={}, Eject={}): {:?}",
                       usage_page, usage, self.fn_down, self.shift_down, self.eject_down, action);
            execute_action(action, self.shift_down);
        }
    }

    /// Tries to trigger a mapping and returns true if an action was executed (should suppress original key)
    pub fn try_trigger_mapping(&mut self, usage_page: u16, usage: u16, value: i32) -> bool {
        if value == 0 {
            return false; // Only trigger and suppress on key-down
        }

        let key = HidKey { usage_page, usage };

        if let Some(action) = self.resolve(key) {
            log::debug!("Triggered mapping for {:04X}:{:04X}, suppressing original", usage_page, usage);
            execute_action(action, self.shift_down);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The shadowing the reporter uses, plus one entry in each other map so the priority rules are
    // covered. Note the Shift prefix: the parser only recognises LEFT_SHIFT+/RIGHT_SHIFT+.
    const MAPPINGS: &str = "\
KEY_C = KEY_D
FN+BACKSPACE = DELETE
LEFT_SHIFT+KEY_A = KEY_B
EJECT+KEY_1 = RUN(\"calc.exe\")
EJECT+FN+KEY_1 = RUN(\"notepad.exe\")
";

    const BACKSPACE: HidKey = HidKey { usage_page: 0x07, usage: 0x2A };
    const KEY_A: HidKey = HidKey { usage_page: 0x07, usage: 0x04 };
    const KEY_1: HidKey = HidKey { usage_page: 0x07, usage: 0x1E };
    const KEY_C: HidKey = HidKey { usage_page: 0x07, usage: 0x06 };

    const FN_STATE: (u16, u16) = (0xFF00, 0x0003);
    const LEFT_SHIFT: (u16, u16) = (0x07, 0xE1);
    const EJECT: (u16, u16) = (0x0C, 0x00B8);

    fn mapper_with_mappings() -> KeyMapper {
        let path = std::env::temp_dir().join(format!(
            "a1314_test_mappings_{}_{:?}.txt",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::write(&path, MAPPINGS).expect("write test mappings");
        let mut mapper = KeyMapper::new();
        mapper.load_mapping_file(&path);
        let _ = std::fs::remove_file(&path);
        mapper
    }

    fn combo(action: Option<&Action>) -> Option<String> {
        match action {
            Some(Action::KeyCombo(combo)) => Some(combo.clone()),
            Some(Action::Run(path)) => Some(format!("RUN({path})")),
            Some(Action::AppCommand(cmd)) => Some(format!("APPCOMMAND({cmd})")),
            None => None,
        }
    }

    fn press(mapper: &mut KeyMapper, key: (u16, u16)) {
        mapper.handle_hid_event(key.0, key.1, 1);
    }

    fn release(mapper: &mut KeyMapper, key: (u16, u16)) {
        mapper.handle_hid_event(key.0, key.1, 0);
    }

    /// The reported case: Fn+Shift+Backspace, where the Shift map has no entry for Backspace.
    /// Before the fix this resolved to nothing, so the daemon never suppressed the key and the
    /// physical Backspace leaked through to Explorer.
    #[test]
    fn shift_falls_back_to_the_fn_mapping() {
        let mut mapper = mapper_with_mappings();
        press(&mut mapper, FN_STATE);
        press(&mut mapper, LEFT_SHIFT);
        assert_eq!(combo(mapper.resolve(BACKSPACE)), Some("DELETE".to_string()));
    }

    #[test]
    fn fn_mapping_still_applies_without_shift() {
        let mut mapper = mapper_with_mappings();
        press(&mut mapper, FN_STATE);
        assert_eq!(combo(mapper.resolve(BACKSPACE)), Some("DELETE".to_string()));
    }

    #[test]
    fn a_shift_mapping_wins_over_the_fn_mapping() {
        let mut mapper = mapper_with_mappings();
        press(&mut mapper, FN_STATE);
        press(&mut mapper, LEFT_SHIFT);
        assert_eq!(combo(mapper.resolve(KEY_A)), Some("KEY_B".to_string()));
    }

    #[test]
    fn eject_and_fn_is_more_specific_than_eject() {
        let mut mapper = mapper_with_mappings();
        press(&mut mapper, EJECT);
        assert_eq!(combo(mapper.resolve(KEY_1)), Some("RUN(calc.exe)".to_string()));
        press(&mut mapper, FN_STATE);
        assert_eq!(combo(mapper.resolve(KEY_1)), Some("RUN(notepad.exe)".to_string()));
    }

    #[test]
    fn plain_mapping_applies_with_no_modifiers() {
        let mapper = mapper_with_mappings();
        assert_eq!(combo(mapper.resolve(KEY_C)), Some("KEY_D".to_string()));
    }

    #[test]
    fn releasing_shift_returns_the_key_to_its_normal_mapping() {
        let mut mapper = mapper_with_mappings();
        press(&mut mapper, FN_STATE);
        press(&mut mapper, LEFT_SHIFT);
        release(&mut mapper, LEFT_SHIFT);
        // No normal mapping for Backspace, and Fn is still down: the Fn map applies.
        assert_eq!(combo(mapper.resolve(BACKSPACE)), Some("DELETE".to_string()));
        release(&mut mapper, FN_STATE);
        assert_eq!(combo(mapper.resolve(BACKSPACE)), None);
    }
}
