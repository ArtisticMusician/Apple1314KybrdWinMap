 <p align="center">
  <img src="RottenAppleIcon.png" width="200" alt="Rotten Apple Logo">
</p>

# Rotten Apple A1314 Daemon 

Developed by Josh McCann | Version 2026.1.31

**Apple A1314 Wireless Keyboard Mapper for Windows**

A professional, high-performance Windows daemon that intercepts raw HID input from the Apple Wireless Keyboard (A1314) and allows seamless key remapping, including full `Fn` and `Eject` modifier support.

[![Version](https://img.shields.io/badge/version-2026.1.31-cyan.svg)](https://github.com/yourusername/a1314_daemon_v20260131)
[![License](https://img.shields.io/badge/license-PERSONAL-dark_green.svg)](LICENSE)
[![Coffee](https://img.shields.io/badge/BUY_ME-COFFEE-blue.svg)](https://buymeacoffee.com/artisticmusician)

---

## ✨ Features

- ✅ **Native Key Suppression** - Uses a low-level keyboard hook to "swallow" original keys during remapping (stops double-keying)
- ✅ **Full Modifier Support** - Custom Fn and Eject modifier detection via vendor-specific HID reports
- ✅ **Self-Contained** - Embedded mapping template and tray icon inside the binary
- ✅ **Zero-Conflict Launches** - Child processes use isolated working directories (no folder locking)
- ✅ **System Tray UI** - Premium tray icon with hot-reload and reset controls
- ✅ **Feedback Loop Prevention** - Advanced tagging to ignore self-injected keystrokes
- ✅ **Hot reload** - Configuration changes apply the instant you save
- ✅ **Professional Logging** - Trace-level HID report inspection and error tracking

---

## 🚀 Quick Start

### Use a Release File

```
- The easiest way to get started is to simply Install a release file.
- EXAMPLE: a1314_daemon_v20260131.exe
```
### **INSTALL INSTRUCTIONS**
```
1. Place the release file in a folder where it will remain permnently.
2. Double Click to install.
3. It will create a file called A1314_mapping.txt
  - This file is where you edit your mappings.
  - Once you edit the file I suggest you back it up somewhere 
  - You can reset the file by right clicking the system tray icon. 
```



### 1. Build Your Own Executable
 - This is not necessary, but you can if you like build your own executable
```bash
cargo build --release
```

### 2. Deploy
Copy the self-contained binary to your preferred location:
- `target\release\a1314_daemon_v20260131.exe`

*Note: The mapping file and icon will be automatically created on first run if missing.*

### 3. Install & Run
Run the installer command:
```bash
a1314_daemon_v20260131.exe --install
```
The daemon will now start automatically every time you log into Windows.

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

### EJECT MAPPING

```
EJECT acts as a modifier key. You can use it to create shortcuts or open programs
```

### Example Mappings

```text
# Map F3 to Task View (Mission Control equivalent)
F3 = WIN+TAB

# Launch Calculator with Eject+1
EJECT+KEY_1 = RUN("calc.exe")

# Map Fn+Delete to forward delete
FN+DELETE = DELETE

# Launch PowerShell with Eject+Fn+1
EJECT+FN+KEY_1 = RUN("powershell.exe")
```

#### Function Keys
- Function Keys default to the Media Functions  
- To use the F1 key you must first press the FN_KEY
- Example: F1=BRIGHTNESS_DOWN, FN+F1=F1

#### Media Keys
```
- F1=BRIGHTNESS_DOWN
- F2=BRIGHTNESS UP
- F3=WIN+TAB
- F4=WIN+S
- F5=WIN+H
- F6=WIN+A
- F7=MEDIA_PREVIOUS
- F8=MEDIA_PLAY_PAUSE
- F9=MEDIA_NEXT
- F10=MUTE
- F11=VOLUME_DOWN
- F12=VOLUME_UP
```

#### Navigation
```
- FN+LEFT_ARROW = HOME
- FN+RIGHT_ARROW = END
- FN+UP_ARROW = PAGE_UP
- FN+DOWN_ARROW = PAGE_DOWN
```

#### Program Launching
++ It is not required to use **EJECT** Key for Program Launching ++
```text
EJECT+KEY_A = RUN("notepad.exe")
EJECT+KEY_M = RUN("C:\Program Files\MyApp\app.exe")
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

- **HID report parsing** is based on Apple A1314 report descriptor (likely will not work for other models)
- **US Layout** This was created for US Layout
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
- Ensure `RottenApple.ico` is in the same directory as the executable
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

## 📄 License

[See License file](LICENSE.md)

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
- I'm just going to be honest with you, You're likely on your own. I'm busy and really just offering this up to help someone out.

---

## 🎯 Roadmap

Future enhancements being considered:

- [ ] Multi-profile support (switch between mapping sets)
- [ ] Visual key tester to identify HID codes

---

**Made with ❤️ for Apple A1314 Wireless Keyboard users on Windows**
