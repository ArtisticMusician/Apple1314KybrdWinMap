// --- START OF FILE tests/unit_tests.rs ---
// Unit tests for individual components

#[cfg(test)]
mod hid_parser_tests {
    use std::collections::HashSet;

    #[test]
    fn test_modifier_byte_parsing() {
        // Test parsing modifier byte (byte 1 of standard keyboard report)
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

        let test_modifier_byte = 0b00000011; // LEFT_CTRL and LEFT_SHIFT pressed
        let mut pressed_modifiers = Vec::new();

        for (bit, &code) in modifier_codes.iter().enumerate() {
            if test_modifier_byte & (1 << bit) != 0 {
                pressed_modifiers.push(code);
            }
        }

        assert_eq!(pressed_modifiers.len(), 2);
        assert!(pressed_modifiers.contains(&0xE0)); // LEFT_CTRL
        assert!(pressed_modifiers.contains(&0xE1)); // LEFT_SHIFT
    }

    #[test]
    fn test_key_state_tracking() {
        // Simulate tracking key press/release states
        let mut current_keys = HashSet::new();
        let mut previous_keys = HashSet::new();

        // Initial state - no keys pressed
        assert_eq!(current_keys.len(), 0);

        // Simulate key press
        current_keys.insert((0x07, 0x04)); // A key
        
        // Detect new key press
        for key in current_keys.iter() {
            if !previous_keys.contains(key) {
                // This is a key-down event
                assert_eq!(*key, (0x07, 0x04));
            }
        }

        // Update previous state
        previous_keys = current_keys.clone();

        // Simulate key release
        current_keys.clear();

        // Detect key release
        for key in previous_keys.iter() {
            if !current_keys.contains(key) {
                // This is a key-up event
                assert_eq!(*key, (0x07, 0x04));
            }
        }
    }

    #[test]
    fn test_report_id_detection() {
        // Test detecting different report IDs
        let reports = vec![
            (vec![0x01, 0x00, 0x00, 0x04], "Standard Keyboard"),
            (vec![0x02, 0xB8, 0x00], "Consumer Control"),
            (vec![0x03, 0xCD, 0x00], "Consumer Control Alt"),
            (vec![0x05, 0x01], "Vendor Specific (Fn)"),
        ];

        for (report, report_type) in reports {
            let report_id = report[0];
            
            match report_id {
                0x01 => assert_eq!(report_type, "Standard Keyboard"),
                0x02 | 0x03 => assert!(report_type.contains("Consumer")),
                0x05 => assert!(report_type.contains("Vendor")),
                _ => panic!("Unknown report ID: {}", report_id),
            }
        }
    }

    #[test]
    fn test_fn_key_state_extraction() {
        // Test extracting Fn key state from vendor-specific report
        let report_fn_pressed = vec![0x05, 0x01]; // Fn pressed (bit 0 set)
        let report_fn_released = vec![0x05, 0x00]; // Fn released

        let fn_state_pressed = (report_fn_pressed[1] & 0x01) != 0;
        let fn_state_released = (report_fn_released[1] & 0x01) != 0;

        assert_eq!(fn_state_pressed, true);
        assert_eq!(fn_state_released, false);
    }

    #[test]
    fn test_consumer_usage_extraction() {
        // Test extracting consumer control usage from report
        let report = vec![0x02, 0xB8, 0x00]; // EJECT key (0x00B8)
        
        if report.len() >= 3 {
            let usage = u16::from_le_bytes([report[1], report[2]]);
            assert_eq!(usage, 0x00B8);
        } else {
            panic!("Report too short");
        }
    }

    #[test]
    fn test_key_rollover_detection() {
        // Test detecting error rollover condition
        const ERROR_ROLLOVER: u8 = 1;
        
        let report_normal = vec![0x01, 0x00, 0x00, 0x04, 0x05, 0x06, 0x00, 0x00]; // A, B, C pressed
        let report_rollover = vec![0x01, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01]; // Error rollover
        
        // Check normal report
        let normal_keys: Vec<u8> = report_normal[3..8]
            .iter()
            .filter(|&&k| k != 0 && k != ERROR_ROLLOVER)
            .copied()
            .collect();
        assert_eq!(normal_keys.len(), 3);
        
        // Check rollover report
        let rollover_detected = report_rollover[3..8]
            .iter()
            .all(|&k| k == ERROR_ROLLOVER);
        assert_eq!(rollover_detected, true);
    }
}

#[cfg(test)]
mod key_mapper_tests {
    use std::collections::HashMap;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct HidKey {
        usage_page: u16,
        usage: u16,
    }

    #[test]
    fn test_normal_mapping() {
        let mut mappings = HashMap::new();
        let key_a = HidKey { usage_page: 0x07, usage: 0x04 };
        
        mappings.insert(key_a, "A".to_string());
        
        assert_eq!(mappings.get(&key_a), Some(&"A".to_string()));
    }

    #[test]
    fn test_fn_mapping() {
        let mut fn_mappings = HashMap::new();
        let key_f1 = HidKey { usage_page: 0x07, usage: 0x3A };
        
        fn_mappings.insert(key_f1, "BRIGHTNESS_DOWN".to_string());
        
        assert_eq!(fn_mappings.get(&key_f1), Some(&"BRIGHTNESS_DOWN".to_string()));
    }

    #[test]
    fn test_shift_mapping() {
        let mut shift_mappings = HashMap::new();
        let key_1 = HidKey { usage_page: 0x07, usage: 0x1E };
        
        shift_mappings.insert(key_1, "!".to_string());
        
        assert_eq!(shift_mappings.get(&key_1), Some(&"!".to_string()));
    }

    #[test]
    fn test_eject_mapping() {
        let mut eject_mappings = HashMap::new();
        let key_1 = HidKey { usage_page: 0x07, usage: 0x1E };
        
        eject_mappings.insert(key_1, "RUN(\"calc.exe\")".to_string());
        
        assert_eq!(eject_mappings.get(&key_1), Some(&"RUN(\"calc.exe\")".to_string()));
    }

    #[test]
    fn test_modifier_state_tracking() {
        let mut fn_down = false;
        let mut shift_down = false;
        let eject_down = false;
        
        assert!(!fn_down && !shift_down && !eject_down);

        // Simulate Fn press
        fn_down = true;
        assert!(fn_down && !shift_down && !eject_down);
        
        // Simulate additional Shift press
        shift_down = true;
        assert!(fn_down && shift_down && !eject_down);
        
        // Simulate Fn release
        fn_down = false;
        assert!(!fn_down && shift_down && !eject_down);
        
        // Simulate all release
        shift_down = false;
        assert!(!fn_down && !shift_down && !eject_down);
    }

    #[test]
    fn test_mapping_priority() {
        // Test that correct mapping is selected based on modifier state
        let key_a = HidKey { usage_page: 0x07, usage: 0x04 };
        
        let mut normal_map = HashMap::new();
        let mut fn_map = HashMap::new();
        let mut shift_map = HashMap::new();
        let mut eject_map = HashMap::new();
        let mut eject_fn_map = HashMap::new();
        
        normal_map.insert(key_a, "A");
        fn_map.insert(key_a, "F1");
        shift_map.insert(key_a, "SHIFT+A");
        eject_map.insert(key_a, "EJECT+A");
        eject_fn_map.insert(key_a, "EJECT+FN+A");
        
        // Test priority selection
        fn select_mapping<'a>(
            key: &HidKey,
            fn_down: bool,
            shift_down: bool,
            eject_down: bool,
            normal: &'a HashMap<HidKey, &str>,
            fn_map: &'a HashMap<HidKey, &str>,
            shift_map: &'a HashMap<HidKey, &str>,
            eject_map: &'a HashMap<HidKey, &str>,
            eject_fn_map: &'a HashMap<HidKey, &str>,
        ) -> Option<&'a str> {
            if eject_down && fn_down {
                eject_fn_map.get(key).copied()
            } else if eject_down {
                eject_map.get(key).copied()
            } else if shift_down {
                shift_map.get(key).copied()
            } else if fn_down {
                fn_map.get(key).copied()
            } else {
                normal.get(key).copied()
            }
        }
        
        assert_eq!(
            select_mapping(&key_a, false, false, false, &normal_map, &fn_map, &shift_map, &eject_map, &eject_fn_map),
            Some("A")
        );
        assert_eq!(
            select_mapping(&key_a, true, false, false, &normal_map, &fn_map, &shift_map, &eject_map, &eject_fn_map),
            Some("F1")
        );
        assert_eq!(
            select_mapping(&key_a, false, true, false, &normal_map, &fn_map, &shift_map, &eject_map, &eject_fn_map),
            Some("SHIFT+A")
        );
        assert_eq!(
            select_mapping(&key_a, false, false, true, &normal_map, &fn_map, &shift_map, &eject_map, &eject_fn_map),
            Some("EJECT+A")
        );
        assert_eq!(
            select_mapping(&key_a, true, false, true, &normal_map, &fn_map, &shift_map, &eject_map, &eject_fn_map),
            Some("EJECT+FN+A")
        );
    }
}

#[cfg(test)]
mod action_executor_tests {
    #[test]
    fn test_key_combo_splitting() {
        let combo = "CTRL+SHIFT+ESC";
        let parts: Vec<&str> = combo.split('+').map(|s| s.trim()).collect();
        
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "CTRL");
        assert_eq!(parts[1], "SHIFT");
        assert_eq!(parts[2], "ESC");
    }

    #[test]
    fn test_modifier_identification() {
        fn is_modifier(key: &str) -> bool {
            matches!(
                key.to_uppercase().as_str(),
                "CTRL" | "CONTROL" | "SHIFT" | "ALT" | "MENU" | "WIN" | "GUI"
            )
        }
        
        assert!(is_modifier("CTRL"));
        assert!(is_modifier("shift"));
        assert!(is_modifier("ALT"));
        assert!(is_modifier("WIN"));
        assert!(!is_modifier("A"));
        assert!(!is_modifier("F1"));
    }

    #[test]
    fn test_virtual_key_lookup() {
        fn get_vk_code(key: &str) -> u16 {
            match key.to_uppercase().as_str() {
                "ESC" | "ESCAPE" => 0x1B,
                "TAB" => 0x09,
                "ENTER" | "RETURN" => 0x0D,
                "A" => 0x41,
                "F1" => 0x70,
                _ => 0,
            }
        }
        
        assert_eq!(get_vk_code("ESC"), 0x1B);
        assert_eq!(get_vk_code("TAB"), 0x09);
        assert_eq!(get_vk_code("A"), 0x41);
        assert_eq!(get_vk_code("UNKNOWN"), 0);
    }

    #[test]
    fn test_run_command_extraction() {
        fn extract_exe_path(action: &str) -> Option<&str> {
            if let Some(rest) = action.strip_prefix("RUN(\"") {
                if let Some(end) = rest.rfind("\")") {
                    return Some(&rest[..end]);
                }
            }
            None
        }
        
        assert_eq!(extract_exe_path("RUN(\"calc.exe\")"), Some("calc.exe"));
        assert_eq!(
            extract_exe_path("RUN(\"C:\\Windows\\notepad.exe\")"),
            Some("C:\\Windows\\notepad.exe")
        );
        assert_eq!(extract_exe_path("WIN+TAB"), None);
    }

    #[test]
    fn test_appcommand_number_extraction() {
        fn extract_command_number(action: &str) -> Option<u32> {
            if let Some(rest) = action.strip_prefix("APPCOMMAND(") {
                if let Some(end) = rest.find(')') {
                    return rest[..end].parse().ok();
                }
            }
            None
        }
        
        assert_eq!(extract_command_number("APPCOMMAND(8)"), Some(8));
        assert_eq!(extract_command_number("APPCOMMAND(46)"), Some(46));
        assert_eq!(extract_command_number("APPCOMMAND(invalid)"), None);
    }

    #[test]
    fn test_key_event_delay() {
        use std::time::{Duration, Instant};
        
        const KEY_EVENT_DELAY_MS: u64 = 1;
        
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(KEY_EVENT_DELAY_MS));
        let elapsed = start.elapsed();
        
        // Allow some tolerance for sleep accuracy
        assert!(elapsed >= Duration::from_millis(KEY_EVENT_DELAY_MS));
        assert!(elapsed < Duration::from_millis(KEY_EVENT_DELAY_MS + 10));
    }
}

#[cfg(test)]
mod variable_maps_tests {
    use std::collections::HashMap;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct HidKey {
        usage_page: u16,
        usage: u16,
    }

    #[test]
    fn test_string_to_hid_key_mapping() {
        let mut map = HashMap::new();
        
        map.insert("KEY_A", HidKey { usage_page: 0x07, usage: 0x0004 });
        map.insert("KEY_B", HidKey { usage_page: 0x07, usage: 0x0005 });
        map.insert("F1", HidKey { usage_page: 0x07, usage: 0x003A });
        map.insert("EJECT", HidKey { usage_page: 0x0C, usage: 0x00B8 });
        
        assert_eq!(
            map.get("KEY_A"),
            Some(&HidKey { usage_page: 0x07, usage: 0x0004 })
        );
        assert_eq!(
            map.get("EJECT"),
            Some(&HidKey { usage_page: 0x0C, usage: 0x00B8 })
        );
        assert_eq!(map.get("UNKNOWN"), None);
    }

    #[test]
    fn test_string_to_action_mapping() {
        let mut map = HashMap::new();
        
        map.insert("WIN+TAB", "KeyCombo:WIN+TAB");
        map.insert("MUTE", "KeyCombo:MUTE");
        map.insert("A", "KeyCombo:A");
        
        assert_eq!(map.get("WIN+TAB"), Some(&"KeyCombo:WIN+TAB"));
        assert_eq!(map.get("A"), Some(&"KeyCombo:A"));
        assert_eq!(map.get("UNKNOWN"), None);
    }

    #[test]
    fn test_usage_page_ranges() {
        // Test that different usage pages are used correctly
        let keyboard_key = HidKey { usage_page: 0x07, usage: 0x04 };
        let consumer_key = HidKey { usage_page: 0x0C, usage: 0xB8 };
        let vendor_key = HidKey { usage_page: 0xFF00, usage: 0x03 };
        
        assert_eq!(keyboard_key.usage_page, 0x07); // Keyboard/Keypad
        assert_eq!(consumer_key.usage_page, 0x0C); // Consumer
        assert_eq!(vendor_key.usage_page, 0xFF00); // Vendor-specific
    }

    #[test]
    fn test_shifted_symbol_mapping() {
        let mut map = HashMap::new();
        
        map.insert("!", "SHIFT+1");
        map.insert("@", "SHIFT+2");
        map.insert("_", "SHIFT+MINUS");
        map.insert("+", "SHIFT+EQUALS");
        
        assert_eq!(map.get("!"), Some(&"SHIFT+1"));
        assert_eq!(map.get("_"), Some(&"SHIFT+MINUS"));
    }
}

#[cfg(test)]
mod file_operations_tests {
    use std::fs;
    use std::path::PathBuf;

    fn setup_test_dir() -> PathBuf {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let test_dir = std::env::temp_dir().join(format!("a1314_test_{}_{}", std::process::id(), now));
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }

    fn cleanup_test_dir(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_file_write_read() {
        let test_dir = setup_test_dir();
        let test_file = test_dir.join("test.txt");
        
        let content = "Test content";
        fs::write(&test_file, content).unwrap();
        
        let read_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(read_content, content);
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_file_modification_detection() {
        let test_dir = setup_test_dir();
        let test_file = test_dir.join("test.txt");
        
        fs::write(&test_file, "Version 1").unwrap();
        let metadata1 = fs::metadata(&test_file).unwrap();
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        fs::write(&test_file, "Version 2").unwrap();
        let metadata2 = fs::metadata(&test_file).unwrap();
        
        // Modified time should be different
        assert_ne!(
            metadata1.modified().unwrap(),
            metadata2.modified().unwrap()
        );
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_path_join() {
        let base = PathBuf::from("C:\\Program Files");
        let joined = base.join("A1314Daemon");
        
        assert!(joined.to_string_lossy().contains("A1314Daemon"));
    }

    #[test]
    fn test_file_exists() {
        let test_dir = setup_test_dir();
        let existing_file = test_dir.join("exists.txt");
        let non_existing_file = test_dir.join("does_not_exist.txt");
        
        fs::write(&existing_file, "content").unwrap();
        
        assert!(existing_file.exists());
        assert!(!non_existing_file.exists());
        
        cleanup_test_dir(&test_dir);
    }
}

#[cfg(test)]
mod logging_tests {
    #[test]
    fn test_log_level_priority() {
        // Test log level ordering (lower number = higher priority)
        const ERROR: u8 = 1;
        const WARN: u8 = 2;
        const INFO: u8 = 3;
        const DEBUG: u8 = 4;
        const TRACE: u8 = 5;
        
        assert!(ERROR < WARN);
        assert!(WARN < INFO);
        assert!(INFO < DEBUG);
        assert!(DEBUG < TRACE);
    }

    #[test]
    fn test_log_message_format() {
        let timestamp = "2024-01-31 14:32:15.123";
        let level = "INFO";
        let message = "Daemon starting";
        
        let formatted = format!("{} [{}] {}", timestamp, level, message);
        
        assert!(formatted.contains(timestamp));
        assert!(formatted.contains(level));
        assert!(formatted.contains(message));
    }
}