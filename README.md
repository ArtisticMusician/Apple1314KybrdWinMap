# A1314 Daemon

**Apple Wireless Keyboard Mapper for Windows**

A Windows daemon that intercepts raw HID input from the Apple Wireless Keyboard (A1314) and allows custom key mapping, especially for Fn-key combinations.

[![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)](https://github.com/yourusername/a1314_daemon)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

---

## ✨ Features

- ✅ Captures raw HID input from A1314 keyboard
- ✅ Supports normal and Fn-modified key mappings
- ✅ Supports SHIFT and EJECT modifiers
- ✅ Remap keys to key combinations (e.g., `WIN+TAB`)
- ✅ Launch programs with keyboard shortcuts
- ✅ **System tray icon** with easy control
- ✅ **Hot reload** - config changes apply instantly
- ✅ **Easy installation** - auto-start with Windows
- ✅ **Reset to defaults** - one click recovery
- ✅ Runs as a background daemon
- ✅ Modular, maintainable architecture
- ✅ Professional logging with configurable levels

---

## 🚀 Quick Start

### 1. Download and Build

```bash
cargo build --release
```

### 2. Copy Files

Copy these files to a permanent location (e.g., `C:\Program Files\A1314Daemon`):
- `target\release\a1314_daemon.exe`
- `A1314_mapping.txt` (optional - will be created automatically)
- `RottenApple_1.ico` (optional - for system tray icon)

### 3. Install

```bash
a1314_daemon.exe --install
```

This adds the daemon to Windows startup. It will run automatically when you log in.

### 4. Run

The daemon starts automatically after installation. You can also run it manually:

```bash
a1314_daemon.exe
```

---

## 🎛️ System Tray Controls

Once running, find the 🍎 icon in your system tray. Right-click for options:

- **Reload Configuration** - Reapply mappings from file
- **Reset to Default Configuration** - Restore original settings
- **Exit** - Stop the daemon

---

## ⚙️ Configuration

### Automatic Hot Reload

Edit `A1314_mapping.txt` and your changes apply **instantly** when you save!

No need to restart the daemon or click reload - just save the file and your new mappings are active.

### Mapping File Format

```text
# Normal key mapping
KEY_NAME = ACTION

# Fn-modified key mapping
FN+KEY_NAME = ACTION

# Shift-modified key mapping
LEFT_SHIFT+KEY_NAME = ACTION
RIGHT_SHIFT+KEY_NAME = ACTION

# Eject-modified key mapping
EJECT+KEY_NAME = ACTION

# Combined modifiers
EJECT+FN+KEY_NAME = ACTION
```

### Example Mappings

```text
# Map F3 to Task View (Mission Control equivalent)
F3 = WIN+TAB

# Launch Calculator with Eject+1
EJECT+KEY_1 = RUN("calc.exe")

# Map Fn+Delete to forward delete
FN+DELETE = DELETE

# Shift+1 produces !
LEFT_SHIFT+KEY_1 = !

# Launch PowerShell with Eject+Fn+1
EJECT+FN+KEY_1 = RUN("powershell.exe")

# Use AppCommand for volume (alternative to media keys)
F10 = APPCOMMAND(8)  # Volume mute
```

### Supported Actions

#### Key Combinations
**Modifiers:** `CTRL`, `SHIFT`, `ALT`, `WIN`

```text
F3 = WIN+TAB
F4 = CTRL+SHIFT+ESC
```

#### Single Keys
**Function Keys:** `F1` through `F12`  
**Letters:** `A` through `Z`  
**Numbers:** `0` through `9`  
**Special:** `ESC`, `TAB`, `SPACE`, `ENTER`, `BACKSPACE`, `DELETE`

#### Media Keys
- `BRIGHTNESS_DOWN`, `BRIGHTNESS_UP`
- `MEDIA_NEXT`, `MEDIA_PREV`, `MEDIA_PLAY_PAUSE`
- `MUTE`, `VOLUME_DOWN`, `VOLUME_UP`

#### Navigation
- `HOME`, `END`, `PAGE_UP`, `PAGE_DOWN`
- `LEFT_ARROW`, `RIGHT_ARROW`, `UP_ARROW`, `DOWN_ARROW`

#### Symbols (US Layout)
- `MINUS`, `EQUALS`, `LEFT_BRACKET`, `RIGHT_BRACKET`
- `SEMICOLON`, `APOSTROPHE`, `GRAVE`, `BACKSLASH`
- `COMMA`, `PERIOD`, `SLASH`

#### Shifted Symbols
- `!`, `@`, `#`, `$`, `%`, `^`, `&`, `*`, `(`, `)`
- `_`, `+`, `{`, `}`, `|`, `:`, `"`, `~`, `<`, `>`, `?`

#### Program Launching
```text
EJECT+KEY_A = RUN("notepad.exe")
EJECT+KEY_M = RUN("C:\Program Files\MyApp\app.exe")
```

#### App Commands
```text
F10 = APPCOMMAND(8)   # Volume Mute
F11 = APPCOMMAND(9)   # Volume Down
F12 = APPCOMMAND(10)  # Volume Up
```

**Note:** App commands may not work in all applications.

---

## 📋 Command Line Options

```bash
# Normal operation - start the daemon
a1314_daemon.exe

# Install to start with Windows
a1314_daemon.exe --install

# Uninstall from Windows startup
a1314_daemon.exe --uninstall

# Show help
a1314_daemon.exe --help
```

---

## 🔧 Advanced Usage

### Logging Levels

Control log verbosity with the `RUST_LOG` environment variable:

```bash
# Normal output (info level) - default
a1314_daemon.exe

# Verbose output (debug level)
set RUST_LOG=debug
a1314_daemon.exe

# Very verbose (trace level - shows HID reports)
set RUST_LOG=trace
a1314_daemon.exe

# Errors only
set RUST_LOG=error
a1314_daemon.exe
```

### Debugging HID Reports

To see what HID reports your keyboard is sending:

1. Set log level to trace:
   ```bash
   set RUST_LOG=trace
   ```
2. Run the daemon
3. Press keys and observe the console output showing raw HID data

This helps you:
- Verify report IDs
- See which bytes contain key data
- Fine-tune the parser for your specific keyboard

---

## 🏗️ Architecture

```
src/
├── main.rs              # Window message loop, system tray, hot reload
├── hid_parser.rs        # Parses A1314 HID reports
├── key_mapper.rs        # Loads mappings and tracks modifier states
├── action_executor.rs   # Executes key combos and launches programs
└── variable_maps.rs     # Hardcoded HID and action mappings
```

**Modular Design:**
- Each module handles a specific responsibility
- Clear separation of concerns
- Easy to extend and maintain
- Thread-safe global state management

---

## 🔄 How It Works

1. **Raw Input Capture:** Registers for raw HID input from the A1314 keyboard
2. **HID Parsing:** Decodes HID reports to extract key events and modifier states (Fn, Shift, Eject)
3. **Mapping Lookup:** Checks if the key has a mapping (with priority for modifiers)
4. **Action Execution:** Sends key combinations via `SendInput` or launches programs
5. **Hot Reload:** File watcher detects config changes and reloads automatically
6. **System Tray:** Provides easy control and status indication

### Modifier Priority

When multiple modifiers are pressed, the daemon uses this priority order:
1. **EJECT+FN** (highest)
2. **EJECT**
3. **SHIFT**
4. **FN**
5. **NORMAL** (lowest)

---

## ⚠️ Known Limitations

- **HID report parsing** is based on Apple A1314 report descriptor (may need tuning for other models)
- **Symbol keys** assume US keyboard layout (symbols may differ on other layouts)
- May require **administrator privileges** for raw input capture on some systems
- The daemon creates an **invisible window** to receive raw input messages
- **App commands** depend on the foreground application supporting them

---

## 🐛 Troubleshooting

### Keys aren't being captured:
1. Ensure keyboard is paired and connected via Bluetooth
2. Run as administrator (right-click → "Run as administrator")
3. Check if `A1314_mapping.txt` exists in the same directory as the executable
4. Verify in Windows Device Manager that the keyboard is recognized

### Fn key not working:
- The Fn key state is tracked via HID usage page `FF00:0003`
- Ensure your keyboard firmware sends this HID code
- Try trace logging to verify: `set RUST_LOG=trace`

### Actions not executing:
- Check the mapping file syntax (no extra spaces, correct format)
- For `RUN()` actions, use full paths with double quotes
- Test key combos work manually first (e.g., `WIN+TAB` opens Task View)
- Check logs for error messages: `set RUST_LOG=debug`

### Configuration not reloading:
- Ensure the file is being saved (not just modified in editor)
- Check logs for file watcher errors
- Try manually clicking "Reload Configuration" in system tray
- Verify file permissions allow reading

### System tray icon not appearing:
- Ensure `RottenApple_1.ico` is in the same directory as the executable
- Check Windows notification area settings
- Try restarting the daemon
- Check logs for tray creation errors

### Installation fails:
- Run as administrator if needed
- Check if you have write permissions to the registry
- Verify the executable path is accessible
- Review logs for specific error messages

### Hot reload not working:
- Ensure the mapping file is in the same directory as the executable
- Save the file completely (some editors use temp files)
- Check file permissions
- Try trace logging to see file events: `set RUST_LOG=trace`

---

## 📝 Changelog

### Version 0.2.0 (Latest)
- ✨ Added system tray icon with menu controls
- ✨ Added automatic hot reload on config file changes
- ✨ Added reset to defaults functionality
- ✨ Added Windows startup installation (`--install`/`--uninstall`)
- ✨ Implemented proper logging framework with levels
- 🐛 Fixed race condition in global state (poison lock recovery)
- 🐛 Fixed AppCommand error handling with better messaging
- 🐛 Eliminated magic numbers with named constants
- 🐛 Improved error messages with helpful hints
- 🐛 Simplified file path logic (always next to executable)
- 🐛 Added configurable key event delays for compatibility
- 📚 Comprehensive documentation updates
- 📚 Added thread safety documentation

### Version 0.1.0
- Initial release
- Basic key mapping functionality
- Fn, Shift, and Eject modifier support
- Program launching capability
- Configuration file support

---

## 🤝 Contributing

Contributions are welcome! Here are some ways you can help:

- Report bugs and issues
- Suggest new features
- Submit pull requests
- Improve documentation
- Test on different keyboard models

---

## 📄 License

This project is provided as-is for educational and personal use.

---

## 🙏 Acknowledgments

- Apple for making great keyboards (even if they need remapping on Windows!)
- The Rust community for excellent crates and tools
- Everyone who contributed feedback and testing

---

## 📞 Support

If you encounter issues:

1. Check the [Troubleshooting](#-troubleshooting) section
2. Enable debug logging: `set RUST_LOG=debug`
3. Review the logs for error messages
4. Open an issue on GitHub with:
   - Your Windows version
   - Keyboard model
   - Log output (with sensitive info removed)
   - Steps to reproduce the issue

---

## 🎯 Roadmap

Future enhancements being considered:

- [ ] Multi-profile support (switch between mapping sets)
- [ ] GUI configuration editor
- [ ] Support for additional Apple keyboard models
- [ ] Macro recording and playback
- [ ] Per-application key mappings
- [ ] Cloud sync for configurations
- [ ] Visual key tester to identify HID codes

---

**Made with ❤️ for Apple keyboard users on Windows**
