# A1314 Daemon

**Apple Wireless Keyboard Mapper for Windows**

A Windows daemon that intercepts raw HID input from the Apple Wireless Keyboard (A1314) and allows custom key mapping, especially for Fn-key combinations.

---

## Features

- ✅ Captures raw HID input from A1314 keyboard
- ✅ Supports normal and Fn-modified key mappings
- ✅ Remap keys to key combinations (e.g., `WIN+TAB`)
- ✅ Launch programs with keyboard shortcuts
- ✅ Runs as a background daemon
- ✅ Modular architecture

---

## Building

```bash
cargo build --release
```

---

## Running

```bash
cargo run --release
```

Or run the compiled binary directly:

```bash
target\release\a1314_daemon.exe
```

---

## Configuration

Edit `A1314_mapping.txt` to customize key mappings.

### Mapping Format

```text
# Normal key mapping
USAGE_PAGE:USAGE = ACTION

# Fn-modified key mapping
FN+USAGE_PAGE:USAGE = ACTION
```

### Examples

```text
# Map F3 to Task View when Fn is pressed
FN+07:003C = WIN+TAB

# Launch an application
FN+07:0020 = RUN("C:\Program Files\MyApp\app.exe")

# Simple key combo
FN+07:003A = CTRL+SHIFT+ESC
```

### Supported Key Combos

**Modifiers:** `CTRL`, `SHIFT`, `ALT`, `WIN`

**Function Keys:** `F1` through `F12`

**Media Keys:** `BRIGHTNESS_DOWN`, `BRIGHTNESS_UP`, `MEDIA_NEXT`, `MEDIA_PREV`, `MEDIA_PLAY_PAUSE`, `MUTE`, `VOLUME_DOWN`, `VOLUME_UP`

**Navigation:** `HOME`, `END`, `PAGE_UP`, `PAGE_DOWN`, `DELETE`

**Special:** `ESC`, `TAB`, `SPACE`, `ENTER`

---

## Architecture

```
src/
├── main.rs              # Window message loop and raw input registration
├── hid_parser.rs        # Parses A1314 HID reports
├── key_mapper.rs        # Loads mappings and tracks Fn state
└── action_executor.rs   # Executes key combos and launches programs
```

**Modular Design:**
- Each module handles a specific responsibility
- Clear separation of concerns
- Easy to extend and maintain

---

## How It Works

1. **Raw Input Capture:** Registers for raw HID input from the A1314 keyboard
2. **HID Parsing:** Decodes HID reports to extract key events and Fn state
3. **Mapping Lookup:** Checks if the key has a mapping (normal or Fn-modified)
4. **Action Execution:** Sends key combinations via `SendInput` or launches programs

---

## Known Limitations

- HID report parsing is based on Apple A1314 report descriptor
- May require administrator privileges for raw input capture
- The daemon creates an invisible window to receive raw input messages
- Currently supports basic HID report formats (may need tuning for edge cases)

---

## Troubleshooting

### Keys aren't being captured:
1. Ensure keyboard is paired and connected via Bluetooth
2. Run as administrator (right-click → "Run as administrator")
3. Verify `A1314_mapping.txt` exists in the same directory as the executable
4. Check Windows Device Manager to confirm the keyboard is recognized

### Fn key not working:
- The Fn key state is tracked via HID usage page `FF00:0003`
- Ensure your keyboard firmware sends this HID code

### Actions not executing:
- Check the mapping file syntax (no extra spaces, correct format)
- For `RUN()` actions, use full paths with double quotes
- Test key combos work manually first (e.g., `WIN+TAB` opens Task View)

### Debugging HID Reports:
To see what HID reports your keyboard is sending:

1. Open `src/hid_parser.rs`
2. Uncomment the debug line (around line 13):
   ```rust
   // eprintln!("HID Report: {:02X?}", report);
   ```
3. Rebuild and run
4. Press keys and observe the console output showing raw HID data

This will help you:
- Verify report IDs
- See which bytes contain key data
- Fine-tune the parser for your specific keyboard

---

## License

This project is provided as-is for educational and personal use.
