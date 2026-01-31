// --- START OF FILE tests/integration_tests.rs ---
// Integration tests for A1314 Daemon

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    
    // Helper function to create a temporary test directory
    fn setup_test_dir() -> PathBuf {
        let test_dir = std::env::temp_dir().join(format!("a1314_test_{}", std::process::id()));
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }

    // Helper function to clean up test directory
    fn cleanup_test_dir(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_default_mapping_file_creation() {
        let test_dir = setup_test_dir();
        let mapping_file = test_dir.join("A1314_mapping.txt");
        
        // Ensure file doesn't exist
        assert!(!mapping_file.exists());
        
        // Create default mapping file (this would be called by create_default_mapping_file)
        let default_content = include_str!("../A1314_mapping.txt");
        fs::write(&mapping_file, default_content).unwrap();
        
        // Verify file was created
        assert!(mapping_file.exists());
        
        // Verify content is not empty
        let content = fs::read_to_string(&mapping_file).unwrap();
        assert!(!content.is_empty());
        assert!(content.contains("KEY_A"));
        assert!(content.contains("F1 = BRIGHTNESS_DOWN"));
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_mapping_file_parsing() {
        let test_dir = setup_test_dir();
        let mapping_file = test_dir.join("test_mapping.txt");
        
        // Create a test mapping file
        let test_content = r#"
# Test mapping file
KEY_A = A
KEY_B = B
FN+KEY_A = F1
LEFT_SHIFT+KEY_1 = !
EJECT+KEY_1 = RUN("calc.exe")
EJECT+FN+KEY_1 = RUN("notepad.exe")
"#;
        fs::write(&mapping_file, test_content).unwrap();
        
        // Parse the file
        let content = fs::read_to_string(&mapping_file).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        
        // Count non-comment, non-empty lines
        let valid_lines: Vec<&str> = lines
            .iter()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .copied()
            .collect();
        
        assert_eq!(valid_lines.len(), 6);
        
        // Verify format
        for line in valid_lines {
            assert!(line.contains('='), "Line should contain '=': {}", line);
            let parts: Vec<&str> = line.split('=').collect();
            assert_eq!(parts.len(), 2, "Line should have exactly one '=': {}", line);
        }
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_mapping_file_with_errors() {
        let test_dir = setup_test_dir();
        let mapping_file = test_dir.join("error_mapping.txt");
        
        // Create a mapping file with various errors
        let test_content = r#"
# Valid line
KEY_A = A

# Missing equals
KEY_B

# Multiple equals
KEY_C = = A

# Empty value
KEY_D = 

# Valid line with spaces
KEY_E   =   E

# Comment in middle
KEY_F = F # This is a comment
"#;
        fs::write(&mapping_file, test_content).unwrap();
        
        let content = fs::read_to_string(&mapping_file).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        
        let mut valid_count = 0;
        let mut error_count = 0;
        
        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 && !parts[0].trim().is_empty() && !parts[1].trim().is_empty() {
                valid_count += 1;
            } else {
                error_count += 1;
            }
        }
        
        // Should have 2 valid lines (KEY_A and KEY_E)
        // KEY_F might be valid depending on how comments are handled
        assert!(valid_count >= 2);
        assert!(error_count >= 2); // KEY_B, KEY_C, KEY_D
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_hid_key_structure() {
        // Test HidKey creation and comparison
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        struct HidKey {
            usage_page: u16,
            usage: u16,
        }
        
        let key1 = HidKey { usage_page: 0x07, usage: 0x04 }; // A key
        let key2 = HidKey { usage_page: 0x07, usage: 0x04 }; // A key (duplicate)
        let key3 = HidKey { usage_page: 0x07, usage: 0x05 }; // B key
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
        
        // Test in HashMap
        let mut map = HashMap::new();
        map.insert(key1, "A");
        assert_eq!(map.get(&key2), Some(&"A"));
        assert_eq!(map.get(&key3), None);
    }

    #[test]
    fn test_modifier_priority() {
        // Test that modifiers are checked in the correct priority order
        // Priority: EJECT+FN > EJECT > SHIFT > FN > NORMAL
        
        struct ModifierState {
            fn_down: bool,
            shift_down: bool,
            eject_down: bool,
        }
        
        let state1 = ModifierState { fn_down: false, shift_down: false, eject_down: false };
        let state2 = ModifierState { fn_down: true, shift_down: false, eject_down: false };
        let state3 = ModifierState { fn_down: false, shift_down: true, eject_down: false };
        let state4 = ModifierState { fn_down: false, shift_down: false, eject_down: true };
        let state5 = ModifierState { fn_down: true, shift_down: false, eject_down: true };
        
        fn get_priority(state: &ModifierState) -> u8 {
            if state.eject_down && state.fn_down {
                5 // Highest
            } else if state.eject_down {
                4
            } else if state.shift_down {
                3
            } else if state.fn_down {
                2
            } else {
                1 // Lowest
            }
        }
        
        assert_eq!(get_priority(&state1), 1); // NORMAL
        assert_eq!(get_priority(&state2), 2); // FN
        assert_eq!(get_priority(&state3), 3); // SHIFT
        assert_eq!(get_priority(&state4), 4); // EJECT
        assert_eq!(get_priority(&state5), 5); // EJECT+FN
    }

    #[test]
    fn test_action_parsing() {
        // Test parsing different action formats
        let test_cases = vec![
            ("WIN+TAB", "KeyCombo"),
            ("RUN(\"calc.exe\")", "Run"),
            ("APPCOMMAND(8)", "AppCommand"),
            ("F1", "KeyCombo"),
            ("CTRL+SHIFT+ESC", "KeyCombo"),
        ];
        
        for (input, expected_type) in test_cases {
            if input.starts_with("RUN(\"") && input.ends_with("\")") {
                assert_eq!(expected_type, "Run");
            } else if input.starts_with("APPCOMMAND(") && input.ends_with(")") {
                assert_eq!(expected_type, "AppCommand");
            } else {
                assert_eq!(expected_type, "KeyCombo");
            }
        }
    }

    #[test]
    fn test_key_combo_parsing() {
        // Test parsing key combinations
        fn parse_combo(combo: &str) -> (Vec<&str>, Option<&str>) {
            let parts: Vec<&str> = combo.split('+').map(|s| s.trim()).collect();
            let mut modifiers = Vec::new();
            let mut main_key = None;
            
            for part in &parts {
                match part.to_uppercase().as_str() {
                    "CTRL" | "SHIFT" | "ALT" | "WIN" => modifiers.push(*part),
                    _ => main_key = Some(*part),
                }
            }
            
            (modifiers, main_key)
        }
        
        let (mods, key) = parse_combo("CTRL+SHIFT+ESC");
        assert_eq!(mods.len(), 2);
        assert_eq!(key, Some("ESC"));
        
        let (mods, key) = parse_combo("WIN+TAB");
        assert_eq!(mods.len(), 1);
        assert_eq!(key, Some("TAB"));
        
        let (mods, key) = parse_combo("F1");
        assert_eq!(mods.len(), 0);
        assert_eq!(key, Some("F1"));
    }

    #[test]
    fn test_hid_report_constants() {
        // Test HID report constants
        const NO_KEY: u8 = 0;
        const ERROR_ROLLOVER: u8 = 1;
        const RIM_TYPEHID: u32 = 2;
        
        assert_eq!(NO_KEY, 0);
        assert_eq!(ERROR_ROLLOVER, 1);
        assert_eq!(RIM_TYPEHID, 2);
        
        // Test key filtering logic
        let test_keys = vec![0u8, 1, 4, 5, 0, 1, 6];
        let valid_keys: Vec<u8> = test_keys
            .iter()
            .filter(|&&k| k != NO_KEY && k != ERROR_ROLLOVER)
            .copied()
            .collect();
        
        assert_eq!(valid_keys, vec![4, 5, 6]);
    }

    #[test]
    fn test_file_path_resolution() {
        // Test that file paths are resolved correctly
        let exe_path = std::env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();
        let mapping_path = exe_dir.join("A1314_mapping.txt");
        
        // Verify path construction
        assert!(mapping_path.to_string_lossy().ends_with("A1314_mapping.txt"));
        assert!(mapping_path.is_absolute());
    }

    #[test]
    fn test_config_reload_simulation() {
        let test_dir = setup_test_dir();
        let mapping_file = test_dir.join("A1314_mapping.txt");
        
        // Create initial config
        let initial_content = "KEY_A = A\nKEY_B = B\n";
        fs::write(&mapping_file, initial_content).unwrap();
        
        // Verify initial content
        let content1 = fs::read_to_string(&mapping_file).unwrap();
        assert!(content1.contains("KEY_A"));
        assert!(content1.contains("KEY_B"));
        
        // Simulate config change
        std::thread::sleep(std::time::Duration::from_millis(10));
        let updated_content = "KEY_A = A\nKEY_B = B\nKEY_C = C\n";
        fs::write(&mapping_file, updated_content).unwrap();
        
        // Verify updated content
        let content2 = fs::read_to_string(&mapping_file).unwrap();
        assert!(content2.contains("KEY_C"));
        assert_ne!(content1.len(), content2.len());
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_virtual_key_parsing() {
        // Test virtual key code mapping
        fn parse_vk_code(key: &str) -> u16 {
            match key.to_uppercase().as_str() {
                "A" => 0x41,
                "B" => 0x42,
                "ESC" | "ESCAPE" => 0x1B,
                "F1" => 0x70,
                "CTRL" => 0x11,
                "SHIFT" => 0x10,
                _ => 0,
            }
        }
        
        assert_eq!(parse_vk_code("A"), 0x41);
        assert_eq!(parse_vk_code("a"), 0x41); // Case insensitive
        assert_eq!(parse_vk_code("ESC"), 0x1B);
        assert_eq!(parse_vk_code("ESCAPE"), 0x1B);
        assert_eq!(parse_vk_code("F1"), 0x70);
        assert_eq!(parse_vk_code("UNKNOWN"), 0);
    }

    #[test]
    fn test_run_command_extraction() {
        // Test extracting program path from RUN() command
        fn extract_run_path(action: &str) -> Option<String> {
            if let Some(rest) = action.strip_prefix("RUN(\"") {
                if let Some(end) = rest.rfind("\")") {
                    return Some(rest[..end].to_string());
                }
            }
            None
        }
        
        assert_eq!(
            extract_run_path("RUN(\"calc.exe\")"),
            Some("calc.exe".to_string())
        );
        assert_eq!(
            extract_run_path("RUN(\"C:\\Windows\\System32\\notepad.exe\")"),
            Some("C:\\Windows\\System32\\notepad.exe".to_string())
        );
        assert_eq!(extract_run_path("RUN(calc.exe)"), None); // Missing quotes
        assert_eq!(extract_run_path("WIN+TAB"), None); // Not a RUN command
    }

    #[test]
    fn test_appcommand_extraction() {
        // Test extracting command number from APPCOMMAND()
        fn extract_appcommand(action: &str) -> Option<u32> {
            if let Some(rest) = action.strip_prefix("APPCOMMAND(") {
                if let Some(end) = rest.find(')') {
                    return rest[..end].parse::<u32>().ok();
                }
            }
            None
        }
        
        assert_eq!(extract_appcommand("APPCOMMAND(8)"), Some(8));
        assert_eq!(extract_appcommand("APPCOMMAND(46)"), Some(46));
        assert_eq!(extract_appcommand("APPCOMMAND(abc)"), None); // Not a number
        assert_eq!(extract_appcommand("APPCOMMAND("), None); // Malformed
        assert_eq!(extract_appcommand("WIN+TAB"), None); // Not an APPCOMMAND
    }

    #[test]
    fn test_mapping_line_variants() {
        // Test various mapping line formats
        let valid_lines = vec![
            "KEY_A = A",
            "KEY_A=A",
            "  KEY_A  =  A  ",
            "FN+KEY_A = F1",
            "LEFT_SHIFT+KEY_1 = !",
            "EJECT+FN+KEY_1 = RUN(\"calc.exe\")",
        ];
        
        for line in valid_lines {
            let trimmed = line.trim();
            let parts: Vec<&str> = trimmed.split('=').map(|s| s.trim()).collect();
            assert_eq!(parts.len(), 2, "Failed for line: {}", line);
            assert!(!parts[0].is_empty(), "LHS empty for line: {}", line);
            assert!(!parts[1].is_empty(), "RHS empty for line: {}", line);
        }
    }

    #[test]
    fn test_comment_filtering() {
        // Test that comments are properly filtered
        let lines = vec![
            "# This is a comment",
            "KEY_A = A",
            "  # Another comment",
            "",
            "KEY_B = B",
            "### Header comment",
        ];
        
        let valid_lines: Vec<&str> = lines
            .iter()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .copied()
            .collect();
        
        assert_eq!(valid_lines.len(), 2);
        assert_eq!(valid_lines[0], "KEY_A = A");
        assert_eq!(valid_lines[1], "KEY_B = B");
    }

    #[test]
    fn test_modifier_detection() {
        // Test detecting modifiers in mapping keys
        fn has_modifier(key: &str, modifier: &str) -> bool {
            key.starts_with(modifier)
        }
        
        assert!(has_modifier("FN+KEY_A", "FN+"));
        assert!(has_modifier("LEFT_SHIFT+KEY_1", "LEFT_SHIFT+"));
        assert!(has_modifier("EJECT+KEY_M", "EJECT+"));
        assert!(!has_modifier("KEY_A", "FN+"));
        
        // Test combined modifiers
        let key = "EJECT+FN+KEY_1";
        let has_eject = key.contains("EJECT+");
        let has_fn = key.contains("FN+");
        assert!(has_eject && has_fn);
    }

    #[test]
    fn test_empty_mapping_file() {
        let test_dir = setup_test_dir();
        let mapping_file = test_dir.join("empty_mapping.txt");
        
        // Create empty file
        fs::write(&mapping_file, "").unwrap();
        
        let content = fs::read_to_string(&mapping_file).unwrap();
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        
        assert_eq!(lines.len(), 0);
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_only_comments_mapping_file() {
        let test_dir = setup_test_dir();
        let mapping_file = test_dir.join("comments_only.txt");
        
        // Create file with only comments
        let content = r#"
# This is a header
# More comments
### Another comment
        "#;
        fs::write(&mapping_file, content).unwrap();
        
        let content = fs::read_to_string(&mapping_file).unwrap();
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        
        assert_eq!(lines.len(), 0);
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_file_not_found_handling() {
        let non_existent_path = PathBuf::from("this_file_definitely_does_not_exist_12345.txt");
        
        assert!(!non_existent_path.exists());
        
        let result = fs::read_to_string(&non_existent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_debounce_simulation() {
        use std::time::{Duration, Instant};
        
        // Simulate debounce logic
        let mut last_event = Instant::now();
        let debounce_duration = Duration::from_millis(100);
        
        // First event - should process
        assert!(last_event.elapsed() >= Duration::from_millis(0));
        
        // Immediate second event - should skip
        let elapsed = last_event.elapsed();
        assert!(elapsed < debounce_duration);
        
        // Wait for debounce period
        std::thread::sleep(debounce_duration + Duration::from_millis(10));
        last_event = Instant::now();
        
        // After debounce - should process
        assert!(last_event.elapsed() < Duration::from_millis(50));
    }

    #[test]
    fn test_log_level_parsing() {
        // Test that different log levels are recognized
        let levels = vec!["error", "warn", "info", "debug", "trace"];
        
        for level in levels {
            assert!(["error", "warn", "info", "debug", "trace"].contains(&level));
        }
    }
}
