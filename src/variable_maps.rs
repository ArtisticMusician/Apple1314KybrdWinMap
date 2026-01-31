// --- START OF FILE src/variable_maps.rs ---
use std::collections::HashMap;
use crate::key_mapper::HidKey;
use crate::action_executor::Action;

// --- Hardcoded mappings from friendly string names to HID keys ---
lazy_static::lazy_static! {
    pub static ref STRING_TO_HID_KEY: HashMap<&'static str, HidKey> = {
        let mut m = HashMap::new();
        // Normal keys
        m.insert("KEY_A", HidKey { usage_page: 0x07, usage: 0x0004 });
        m.insert("KEY_B", HidKey { usage_page: 0x07, usage: 0x0005 });
        m.insert("KEY_C", HidKey { usage_page: 0x07, usage: 0x0006 });
        m.insert("KEY_D", HidKey { usage_page: 0x07, usage: 0x0007 });
        m.insert("KEY_E", HidKey { usage_page: 0x07, usage: 0x0008 });
        m.insert("KEY_F", HidKey { usage_page: 0x07, usage: 0x0009 });
        m.insert("KEY_G", HidKey { usage_page: 0x07, usage: 0x000A });
        m.insert("KEY_H", HidKey { usage_page: 0x07, usage: 0x000B });
        m.insert("KEY_I", HidKey { usage_page: 0x07, usage: 0x000C });
        m.insert("KEY_J", HidKey { usage_page: 0x07, usage: 0x000D });
        m.insert("KEY_K", HidKey { usage_page: 0x07, usage: 0x000E });
        m.insert("KEY_L", HidKey { usage_page: 0x07, usage: 0x000F });
        m.insert("KEY_M", HidKey { usage_page: 0x07, usage: 0x0010 });
        m.insert("KEY_N", HidKey { usage_page: 0x07, usage: 0x0011 });
        m.insert("KEY_O", HidKey { usage_page: 0x07, usage: 0x0012 });
        m.insert("KEY_P", HidKey { usage_page: 0x07, usage: 0x0013 });
        m.insert("KEY_Q", HidKey { usage_page: 0x07, usage: 0x0014 });
        m.insert("KEY_R", HidKey { usage_page: 0x07, usage: 0x0015 });
        m.insert("KEY_S", HidKey { usage_page: 0x07, usage: 0x0016 });
        m.insert("KEY_T", HidKey { usage_page: 0x07, usage: 0x0017 });
        m.insert("KEY_U", HidKey { usage_page: 0x07, usage: 0x0018 });
        m.insert("KEY_V", HidKey { usage_page: 0x07, usage: 0x0019 });
        m.insert("KEY_W", HidKey { usage_page: 0x07, usage: 0x001A });
        m.insert("KEY_X", HidKey { usage_page: 0x07, usage: 0x001B });
        m.insert("KEY_Y", HidKey { usage_page: 0x07, usage: 0x001C });
        m.insert("KEY_Z", HidKey { usage_page: 0x07, usage: 0x001D });

        // Number row
        m.insert("KEY_1", HidKey { usage_page: 0x07, usage: 0x001E });
        m.insert("KEY_2", HidKey { usage_page: 0x07, usage: 0x001F });
        m.insert("KEY_3", HidKey { usage_page: 0x07, usage: 0x0020 });
        m.insert("KEY_4", HidKey { usage_page: 0x07, usage: 0x0021 });
        m.insert("KEY_5", HidKey { usage_page: 0x07, usage: 0x0022 });
        m.insert("KEY_6", HidKey { usage_page: 0x07, usage: 0x0023 });
        m.insert("KEY_7", HidKey { usage_page: 0x07, usage: 0x0024 });
        m.insert("KEY_8", HidKey { usage_page: 0x07, usage: 0x0025 });
        m.insert("KEY_9", HidKey { usage_page: 0x07, usage: 0x0026 });
        m.insert("KEY_0", HidKey { usage_page: 0x07, usage: 0x0027 });

        // Basic controls
        m.insert("ENTER", HidKey { usage_page: 0x07, usage: 0x0028 });
        m.insert("ESCAPE", HidKey { usage_page: 0x07, usage: 0x0029 });
        m.insert("BACKSPACE", HidKey { usage_page: 0x07, usage: 0x002A });
        m.insert("TAB", HidKey { usage_page: 0x07, usage: 0x002B });
        m.insert("SPACE", HidKey { usage_page: 0x07, usage: 0x002C });

        // Function keys
        m.insert("F1", HidKey { usage_page: 0x07, usage: 0x003A });
        m.insert("F2", HidKey { usage_page: 0x07, usage: 0x003B });
        m.insert("F3", HidKey { usage_page: 0x07, usage: 0x003C });
        m.insert("F4", HidKey { usage_page: 0x07, usage: 0x003D });
        m.insert("F5", HidKey { usage_page: 0x07, usage: 0x003E });
        m.insert("F6", HidKey { usage_page: 0x07, usage: 0x003F });
        m.insert("F7", HidKey { usage_page: 0x07, usage: 0x0040 });
        m.insert("F8", HidKey { usage_page: 0x07, usage: 0x0041 });
        m.insert("F9", HidKey { usage_page: 0x07, usage: 0x0042 });
        m.insert("F10", HidKey { usage_page: 0x07, usage: 0x0043 });
        m.insert("F11", HidKey { usage_page: 0x07, usage: 0x0044 });
        m.insert("F12", HidKey { usage_page: 0x07, usage: 0x0045 });

        // Arrows
        m.insert("RIGHT_ARROW", HidKey { usage_page: 0x07, usage: 0x004F });
        m.insert("LEFT_ARROW", HidKey { usage_page: 0x07, usage: 0x0050 });
        m.insert("DOWN_ARROW", HidKey { usage_page: 0x07, usage: 0x0051 });
        m.insert("UP_ARROW", HidKey { usage_page: 0x07, usage: 0x0052 });

        // Modifiers (These are used internally by the Raw Input Handler, not typically mapped by user directly)
        m.insert("LEFT_CTRL", HidKey { usage_page: 0x07, usage: 0x00E0 });
        m.insert("LEFT_SHIFT", HidKey { usage_page: 0x07, usage: 0x00E1 });
        m.insert("LEFT_ALT", HidKey { usage_page: 0x07, usage: 0x00E2 });
        m.insert("LEFT_GUI", HidKey { usage_page: 0x07, usage: 0x00E3 });
        m.insert("RIGHT_CTRL", HidKey { usage_page: 0x07, usage: 0x00E4 });
        m.insert("RIGHT_SHIFT", HidKey { usage_page: 0x07, usage: 0x00E5 });
        m.insert("RIGHT_ALT", HidKey { usage_page: 0x07, usage: 0x00E6 });
        m.insert("RIGHT_GUI", HidKey { usage_page: 0x07, usage: 0x00E7 });

        // Consumer/media keys
        m.insert("BRIGHTNESS_DOWN", HidKey { usage_page: 0x0C, usage: 0x006F });
        m.insert("BRIGHTNESS_UP", HidKey { usage_page: 0x0C, usage: 0x0070 });
        m.insert("MEDIA_NEXT", HidKey { usage_page: 0x0C, usage: 0x00B3 });
        m.insert("MEDIA_PREV", HidKey { usage_page: 0x0C, usage: 0x00B4 });
        m.insert("EJECT", HidKey { usage_page: 0x0C, usage: 0x00B8 }); // EJECT key
        m.insert("MEDIA_PLAY_PAUSE", HidKey { usage_page: 0x0C, usage: 0x00CD });
        m.insert("MUTE", HidKey { usage_page: 0x0C, usage: 0x00E2 });
        m.insert("VOLUME_UP", HidKey { usage_page: 0x0C, usage: 0x00E9 });
        m.insert("VOLUME_DOWN", HidKey { usage_page: 0x0C, usage: 0x00EA });

        // Fn state (Apple vendor page)
        m.insert("FN_STATE", HidKey { usage_page: 0xFF00, usage: 0x0003 });
        m
    };
}

// --- Hardcoded mappings from friendly string names to Actions for RHS ---
lazy_static::lazy_static! {
    pub static ref STRING_TO_ACTION: HashMap<&'static str, Action> = {
        let mut m = HashMap::new();
        // Common Action::KeyCombo strings
        m.insert("WIN+TAB", Action::KeyCombo("WIN+TAB".to_string()));
        m.insert("WIN+S", Action::KeyCombo("WIN+S".to_string()));
        m.insert("WIN+H", Action::KeyCombo("WIN+H".to_string()));
        m.insert("WIN+A", Action::KeyCombo("WIN+A".to_string()));
        m.insert("DELETE", Action::KeyCombo("DELETE".to_string()));
        m.insert("HOME", Action::KeyCombo("HOME".to_string()));
        m.insert("END", Action::KeyCombo("END".to_string()));
        m.insert("PAGE_UP", Action::KeyCombo("PAGE_UP".to_string()));
        m.insert("PAGE_DOWN", Action::KeyCombo("PAGE_DOWN".to_string()));
        m.insert("MUTE", Action::KeyCombo("MUTE".to_string()));
        m.insert("BRIGHTNESS_DOWN", Action::KeyCombo("BRIGHTNESS_DOWN".to_string()));
        m.insert("BRIGHTNESS_UP", Action::KeyCombo("BRIGHTNESS_UP".to_string()));
        m.insert("MEDIA_NEXT", Action::KeyCombo("MEDIA_NEXT".to_string()));
        m.insert("MEDIA_PREV", Action::KeyCombo("MEDIA_PREV".to_string()));
        m.insert("MEDIA_PLAY_PAUSE", Action::KeyCombo("MEDIA_PLAY_PAUSE".to_string()));
        m.insert("VOLUME_UP", Action::KeyCombo("VOLUME_UP".to_string()));
        m.insert("VOLUME_DOWN", Action::KeyCombo("VOLUME_DOWN".to_string()));
        
        // Add all single character/number/symbol keys if they can appear on RHS
        // This is important if you want to map `FN+KEY_1 = A` for instance.
        m.insert("A", Action::KeyCombo("A".to_string()));
        m.insert("B", Action::KeyCombo("B".to_string()));
        m.insert("C", Action::KeyCombo("C".to_string()));
        m.insert("D", Action::KeyCombo("D".to_string()));
        m.insert("E", Action::KeyCombo("E".to_string()));
        m.insert("F", Action::KeyCombo("F".to_string()));
        m.insert("G", Action::KeyCombo("G".to_string()));
        m.insert("H", Action::KeyCombo("H".to_string()));
        m.insert("I", Action::KeyCombo("I".to_string()));
        m.insert("J", Action::KeyCombo("J".to_string()));
        m.insert("K", Action::KeyCombo("K".to_string()));
        m.insert("L", Action::KeyCombo("L".to_string()));
        m.insert("M", Action::KeyCombo("M".to_string()));
        m.insert("N", Action::KeyCombo("N".to_string()));
        m.insert("O", Action::KeyCombo("O".to_string()));
        m.insert("P", Action::KeyCombo("P".to_string()));
        m.insert("Q", Action::KeyCombo("Q".to_string()));
        m.insert("R", Action::KeyCombo("R".to_string()));
        m.insert("S", Action::KeyCombo("S".to_string()));
        m.insert("T", Action::KeyCombo("T".to_string()));
        m.insert("U", Action::KeyCombo("U".to_string()));
        m.insert("V", Action::KeyCombo("V".to_string()));
        m.insert("W", Action::KeyCombo("W".to_string()));
        m.insert("X", Action::KeyCombo("X".to_string()));
        m.insert("Y", Action::KeyCombo("Y".to_string()));
        m.insert("Z", Action::KeyCombo("Z".to_string()));
        m.insert("0", Action::KeyCombo("0".to_string()));
        m.insert("1", Action::KeyCombo("1".to_string()));
        m.insert("2", Action::KeyCombo("2".to_string()));
        m.insert("3", Action::KeyCombo("3".to_string()));
        m.insert("4", Action::KeyCombo("4".to_string()));
        m.insert("5", Action::KeyCombo("5".to_string()));
        m.insert("6", Action::KeyCombo("6".to_string()));
        m.insert("7", Action::KeyCombo("7".to_string()));
        m.insert("8", Action::KeyCombo("8".to_string()));
        m.insert("9", Action::KeyCombo("9".to_string()));
        m.insert("ENTER", Action::KeyCombo("ENTER".to_string()));
        m.insert("ESCAPE", Action::KeyCombo("ESCAPE".to_string()));
        m.insert("BACKSPACE", Action::KeyCombo("BACKSPACE".to_string()));
        m.insert("TAB", Action::KeyCombo("TAB".to_string()));
        m.insert("SPACE", Action::KeyCombo("SPACE".to_string()));
        m.insert("F1", Action::KeyCombo("F1".to_string()));
        m.insert("F2", Action::KeyCombo("F2".to_string()));
        m.insert("F3", Action::KeyCombo("F3".to_string()));
        m.insert("F4", Action::KeyCombo("F4".to_string()));
        m.insert("F5", Action::KeyCombo("F5".to_string()));
        m.insert("F6", Action::KeyCombo("F6".to_string()));
        m.insert("F7", Action::KeyCombo("F7".to_string()));
        m.insert("F8", Action::KeyCombo("F8".to_string()));
        m.insert("F9", Action::KeyCombo("F9".to_string()));
        m.insert("F10", Action::KeyCombo("F10".to_string()));
        m.insert("F11", Action::KeyCombo("F11".to_string()));
        m.insert("F12", Action::KeyCombo("F12".to_string()));
        m.insert("RIGHT_ARROW", Action::KeyCombo("RIGHT_ARROW".to_string()));
        m.insert("LEFT_ARROW", Action::KeyCombo("LEFT_ARROW".to_string()));
        m.insert("DOWN_ARROW", Action::KeyCombo("DOWN_ARROW".to_string()));
        m.insert("UP_ARROW", Action::KeyCombo("UP_ARROW".to_string()));
        
        m
    };
}