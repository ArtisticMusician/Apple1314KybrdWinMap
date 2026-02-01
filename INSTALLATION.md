# Quick Installation Guide - A1314 Daemon v1.0.0

## 📦 What You Need

All files should be in the same directory:
- `a1314_daemon.exe` (the compiled binary)
- `A1314_mapping.txt` (will be created automatically if missing)
- `RottenApple.ico` (optional - for system tray icon)

## 🚀 Installation Steps

### Option 1: Automatic Installation (Recommended)

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Copy files to a permanent location:**
   ```bash
   # Create directory (choose your preferred location)
   mkdir "C:\Program Files\A1314Daemon"
   
   # Copy files
   copy target\release\a1314_daemon.exe "C:\Program Files\A1314Daemon\"
   copy RottenApple.ico "C:\Program Files\A1314Daemon\"
   copy A1314_mapping.txt "C:\Program Files\A1314Daemon\"  # Optional
   ```

3. **Install to Windows startup:**
   ```bash
   cd "C:\Program Files\A1314Daemon"
   a1314_daemon.exe --install
   ```

4. **Start the daemon:**
   ```bash
   a1314_daemon.exe
   ```

   Or just log out and log back in (it will start automatically).

### Option 2: Manual Run (No Installation)

Just run the executable:
```bash
cd target\release
a1314_daemon.exe
```

The daemon will run until you close it. Use the system tray icon to exit.

## ✅ Verify Installation

1. **Check system tray** - Look for the 🍎 icon
2. **Right-click the icon** - Menu should appear with:
   - Reload Configuration
   - Reset to Default Configuration
   - Exit
3. **Test a key mapping** - Try pressing a mapped key (e.g., F3 for Task View)

## 🔧 Configuration

### Edit Mappings

1. Open `A1314_mapping.txt` in any text editor
2. Make your changes
3. Save the file
4. **Changes apply instantly!** (no need to reload)

### Reset to Defaults

If you mess up your configuration:
1. Right-click system tray icon
2. Select "Reset to Default Configuration"
3. Mappings are restored instantly

## 🗑️ Uninstallation

### Remove from Windows Startup

```bash
a1314_daemon.exe --uninstall
```

### Complete Removal

1. Exit the daemon (right-click system tray icon → Exit)
2. Run uninstall command (above)
3. Delete the folder containing the executable

## ⚙️ Advanced Configuration

### Change Log Level

**Windows Command Prompt:**
```cmd
set RUST_LOG=debug
a1314_daemon.exe
```

**PowerShell:**
```powershell
$env:RUST_LOG="debug"
.\a1314_daemon.exe
```

**Permanently (System Environment Variable):**
1. Right-click "This PC" → Properties
2. Advanced system settings → Environment Variables
3. Add new User variable:
   - Name: `RUST_LOG`
   - Value: `debug` (or `info`, `trace`, `error`)

### Log Levels

- `error` - Only errors
- `warn` - Errors and warnings
- `info` - Normal operation (default)
- `debug` - Detailed debugging
- `trace` - Very verbose (includes HID reports)

## 🐛 Troubleshooting

### "Access Denied" when installing
- Run Command Prompt as Administrator
- Or install to user directory (e.g., `%LOCALAPPDATA%\A1314Daemon`)

### System tray icon not appearing
- Ensure `RottenApple.ico` is in the same directory as the executable
- Check Windows notification area settings (taskbar settings)
- Try restarting the daemon

### Keyboard not responding
- Check Bluetooth connection
- Verify keyboard is paired in Windows settings
- Try running as Administrator
- Check logs: `set RUST_LOG=debug`

### Configuration not loading
- Ensure `A1314_mapping.txt` is in same directory as exe
- Check file for syntax errors
- View logs for error messages
- Try "Reset to Default Configuration"

## 📁 Recommended Directory Structure

```
C:\Program Files\A1314Daemon\
├── a1314_daemon.exe
├── A1314_mapping.txt
└── RottenApple.ico
```

Or for user-level installation:

```
%LOCALAPPDATA%\A1314Daemon\
├── a1314_daemon.exe
├── A1314_mapping.txt
└── RottenApple.ico
```

## 🔒 Permissions

The daemon needs:
- ✅ Read access to `A1314_mapping.txt`
- ✅ Raw input device access (usually granted automatically)
- ✅ Registry write access for `--install` (user-level only)
- ⚠️ May need Administrator rights on some systems

## 📝 Tips

1. **Keep backups** of your custom `A1314_mapping.txt`
2. **Test mappings** before adding complex ones
3. **Use debug logging** when troubleshooting
4. **Start simple** - add mappings gradually
5. **Read the README** for complete documentation

## 🎯 Next Steps

After installation:
1. ✅ Verify system tray icon appears
2. ✅ Test default mappings (e.g., F3 for Task View)
3. ✅ Customize mappings in `A1314_mapping.txt`
4. ✅ Test your custom mappings
5. ✅ Enjoy your remapped keyboard! 🎉

## 💡 Quick Reference Card

| Action | Command / Method |
|--------|-----------------|
| Install | `a1314_daemon.exe --install` |
| Uninstall | `a1314_daemon.exe --uninstall` |
| Run manually | `a1314_daemon.exe` |
| Edit config | Open `A1314_mapping.txt` in text editor |
| Reload config | Save file (auto-reloads) or right-click tray → Reload |
| Reset config | Right-click tray → Reset to Default |
| Exit | Right-click tray → Exit |
| Help | `a1314_daemon.exe --help` |
| Debug mode | `set RUST_LOG=debug` then run |

## ❓ Getting Help

If you encounter issues:
1. Check logs: `set RUST_LOG=debug` and review output
2. Read the [Troubleshooting](#-troubleshooting) section
3. Consult the full README.md
4. Check existing GitHub issues
5. Open a new issue with:
   - Your Windows version
   - Keyboard model
   - Log output (remove sensitive info)
   - Steps to reproduce

---

**Happy typing! 🍎⌨️**
