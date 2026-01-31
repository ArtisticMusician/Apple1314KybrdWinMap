# Build and Deployment Guide - A1314 Daemon v0.2.0

Complete guide for building, testing, and deploying the A1314 Daemon.

---

## 📋 Prerequisites

### Required Tools
- **Rust** (latest stable) - Download from https://rustup.rs/
- **Windows 10/11** - 64-bit
- **Visual Studio Build Tools** or **Visual Studio** with C++ support

### Verify Installation
```bash
rustc --version
cargo --version
```

---

## 🔨 Building from Source

### 1. Clone/Download Project

Ensure you have all source files in your project directory:
```
a1314_daemon/
├── Cargo.toml
├── A1314_mapping.txt
├── RottenApple.ico
└── src/
    ├── main.rs
    ├── hid_parser.rs
    ├── key_mapper.rs
    ├── action_executor.rs
    └── variable_maps.rs
```

### 2. Build Debug Version (for testing)

```bash
cargo build
```

Output: `target/debug/a1314_daemon.exe`

**Use for:**
- Development
- Testing new features
- Debugging issues

### 3. Build Release Version (for deployment)

```bash
cargo build --release
```

Output: `target/release/a1314_daemon.exe`

**Use for:**
- Production deployment
- Daily use
- Distribution to others

**Benefits:**
- ~10x faster execution
- Optimized binary size
- No debug symbols
- Production-ready

---

## 🧪 Testing

### Basic Functionality Test

1. **Build debug version:**
   ```bash
   cargo build
   ```

2. **Run with verbose logging:**
   ```bash
   set RUST_LOG=debug
   target\debug\a1314_daemon.exe
   ```

3. **Verify:**
   - ✅ Console shows startup messages
   - ✅ System tray icon appears
   - ✅ Configuration loads without errors
   - ✅ Key presses are detected (check logs)
   - ✅ Mapped actions execute correctly

### Hot Reload Test

1. **Start daemon** (debug or release)
2. **Open A1314_mapping.txt** in text editor
3. **Add a test mapping:**
   ```
   # Test mapping
   F12 = RUN("notepad.exe")
   ```
4. **Save file**
5. **Check logs** - should see "Mapping file changed, reloading..."
6. **Press F12** - Notepad should launch

### System Tray Test

1. **Locate tray icon** in notification area
2. **Right-click icon** - menu should appear
3. **Test each menu item:**
   - ✅ Reload Configuration - should reload without errors
   - ✅ Reset to Default - should restore defaults and reload
   - ✅ Exit - should cleanly shut down daemon

### Installation Test

1. **Test install command:**
   ```bash
   target\release\a1314_daemon.exe --install
   ```
2. **Verify registry entry:**
   ```bash
   reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v A1314Daemon
   ```
3. **Log out and log in** - daemon should start automatically
4. **Test uninstall:**
   ```bash
   target\release\a1314_daemon.exe --uninstall
   ```
5. **Verify registry entry removed**

---

## 📦 Deployment Package

### Required Files

Create a deployment package with these files:

```
A1314_Daemon_v0.2.0/
├── a1314_daemon.exe        (from target/release/)
├── A1314_mapping.txt       (default config)
├── RottenApple.ico       (system tray icon)
├── README.md               (documentation)
├── INSTALLATION.md         (setup guide)
└── CHANGELOG.md            (version history)
```

### Optional Files

```
├── LICENSE                 (if distributing)
└── example_configs/        (community examples)
    ├── gaming.txt
    ├── programming.txt
    └── media_center.txt
```

### Creating Distribution Archive

**Using PowerShell:**
```powershell
# Create directory
New-Item -ItemType Directory -Path "A1314_Daemon_v0.2.0"

# Copy files
Copy-Item target\release\a1314_daemon.exe A1314_Daemon_v0.2.0\
Copy-Item A1314_mapping.txt A1314_Daemon_v0.2.0\
Copy-Item RottenApple.ico A1314_Daemon_v0.2.0\
Copy-Item README.md A1314_Daemon_v0.2.0\
Copy-Item INSTALLATION.md A1314_Daemon_v0.2.0\
Copy-Item CHANGELOG.md A1314_Daemon_v0.2.0\

# Create ZIP
Compress-Archive -Path A1314_Daemon_v0.2.0 -DestinationPath A1314_Daemon_v0.2.0.zip
```

---

## 🚀 Deployment Options

### Option 1: User-Level Installation (Recommended)

**Location:** `%LOCALAPPDATA%\A1314Daemon\`

**Advantages:**
- No admin rights needed
- Per-user configuration
- Isolated from other users
- Easy to uninstall

**Steps:**
```powershell
# Create directory
New-Item -ItemType Directory -Path "$env:LOCALAPPDATA\A1314Daemon"

# Copy files
Copy-Item A1314_Daemon_v0.2.0\* "$env:LOCALAPPDATA\A1314Daemon\"

# Install
& "$env:LOCALAPPDATA\A1314Daemon\a1314_daemon.exe" --install

# Run
& "$env:LOCALAPPDATA\A1314Daemon\a1314_daemon.exe"
```

### Option 2: System-Wide Installation

**Location:** `C:\Program Files\A1314Daemon\`

**Advantages:**
- Available to all users
- Professional deployment
- Standard Windows location

**Requirements:**
- Administrator rights
- Manual registry setup or Group Policy

**Steps:**
```powershell
# Run PowerShell as Administrator
# Create directory
New-Item -ItemType Directory -Path "C:\Program Files\A1314Daemon"

# Copy files
Copy-Item A1314_Daemon_v0.2.0\* "C:\Program Files\A1314Daemon\"

# Install for current user
& "C:\Program Files\A1314Daemon\a1314_daemon.exe" --install

# Run
& "C:\Program Files\A1314Daemon\a1314_daemon.exe"
```

### Option 3: Portable Installation

**Location:** Any directory (e.g., USB drive, cloud folder)

**Advantages:**
- No installation needed
- Portable between computers
- Easy to update

**Disadvantages:**
- Won't auto-start with Windows
- Manual startup required

**Steps:**
1. Extract ZIP to desired location
2. Run `a1314_daemon.exe` manually
3. Optionally run `--install` to add to startup

---

## 🔄 Update Procedure

### Updating Existing Installation

1. **Exit current daemon** (right-click tray → Exit)
2. **Backup configuration:**
   ```bash
   copy A1314_mapping.txt A1314_mapping.txt.backup
   ```
3. **Replace executable:**
   ```bash
   copy /Y new_version\a1314_daemon.exe .
   ```
4. **Copy new icon (if updated):**
   ```bash
   copy /Y new_version\RottenApple.ico .
   ```
5. **Start new version:**
   ```bash
   a1314_daemon.exe
   ```
6. **Verify:** Check system tray and test mappings

**Note:** Configuration file (`A1314_mapping.txt`) is preserved.

---

## 🏢 Enterprise Deployment

### Using Group Policy

1. **Copy to network share:**
   ```
   \\fileserver\software\A1314Daemon\
   ```

2. **Create startup script:**
   ```batch
   @echo off
   set INSTALL_PATH=%LOCALAPPDATA%\A1314Daemon
   
   if not exist "%INSTALL_PATH%" (
       mkdir "%INSTALL_PATH%"
   )
   
   xcopy /Y /E "\\fileserver\software\A1314Daemon\*" "%INSTALL_PATH%\"
   
   start "" "%INSTALL_PATH%\a1314_daemon.exe"
   ```

3. **Deploy via GPO:**
   - Computer Configuration → Windows Settings → Scripts → Startup
   - Or User Configuration → Windows Settings → Scripts → Logon

### Using SCCM/Intune

Create deployment package with:
- Detection method: Check for `%LOCALAPPDATA%\A1314Daemon\a1314_daemon.exe`
- Installation command: `install.bat` (copy files + run `--install`)
- Uninstall command: `uninstall.bat` (run `--uninstall` + delete files)

---

## 🔍 Verification Checklist

After deployment, verify:

- [ ] Executable runs without errors
- [ ] System tray icon appears
- [ ] Configuration file loads successfully
- [ ] Key mappings work as expected
- [ ] Hot reload functions when editing config
- [ ] Reset to defaults works
- [ ] Installation to startup successful (if used)
- [ ] Uninstallation removes all traces
- [ ] Logs show no errors (set RUST_LOG=debug)

---

## 🐛 Common Build Issues

### Issue: `linking with link.exe failed`

**Cause:** Missing Visual Studio Build Tools

**Solution:**
1. Install Visual Studio Build Tools
2. Or install Visual Studio Community with C++ support
3. Restart terminal and rebuild

### Issue: `error: could not compile`

**Cause:** Syntax error or dependency issue

**Solution:**
```bash
# Clean build
cargo clean

# Update dependencies
cargo update

# Rebuild
cargo build --release
```

### Issue: Icon not embedding

**Cause:** Icon file not found

**Solution:**
- Ensure `RottenApple.ico` is in project root
- Check file name matches exactly (case-sensitive on some systems)

### Issue: Registry access denied

**Cause:** Insufficient permissions

**Solution:**
- Run Command Prompt as Administrator
- Or use user-level installation directory

---

## 📊 Build Optimization

### Reducing Binary Size

Add to `Cargo.toml`:
```toml
[profile.release]
opt-level = 'z'     # Optimize for size
lto = true          # Enable Link Time Optimization
codegen-units = 1   # Reduce parallel codegen (smaller binary)
panic = 'abort'     # Remove unwind info
strip = true        # Strip symbols
```

**Result:** ~30% smaller binary

### Improving Build Speed

```bash
# Use more CPU cores
set CARGO_BUILD_JOBS=8

# Cache dependencies
cargo install sccache
set RUSTC_WRAPPER=sccache
```

---

## 🔐 Security Considerations

### Before Distribution

1. **Scan for malware** (antivirus/Windows Defender)
2. **Test in sandbox** environment first
3. **Sign executable** (optional, for enterprise)
4. **Document** all dependencies and versions
5. **Provide checksums** (SHA-256) of binaries

### Generating Checksums

```powershell
# PowerShell
Get-FileHash a1314_daemon.exe -Algorithm SHA256
```

```bash
# Linux/WSL
sha256sum a1314_daemon.exe
```

---

## 📝 Distribution Checklist

Before releasing:

- [ ] All features tested and working
- [ ] Documentation up to date
- [ ] Version number bumped in Cargo.toml
- [ ] CHANGELOG.md updated
- [ ] No debug print statements left in code
- [ ] Release build tested on clean machine
- [ ] Icon file included
- [ ] Default config included
- [ ] Installation guide included
- [ ] Checksums generated
- [ ] Release notes written

---

## 🎯 Quick Reference

| Task | Command |
|------|---------|
| Build debug | `cargo build` |
| Build release | `cargo build --release` |
| Run tests | `cargo test` |
| Check code | `cargo check` |
| Format code | `cargo fmt` |
| Lint code | `cargo clippy` |
| Clean build | `cargo clean` |
| Update deps | `cargo update` |

---

## 💡 Tips for Developers

1. **Use debug builds** during development (faster compile)
2. **Use release builds** for testing actual performance
3. **Enable logging** to debug issues
4. **Test on clean VM** before distributing
5. **Keep backups** of working configurations
6. **Document changes** in CHANGELOG.md
7. **Test updates** before deploying to users

---

**Ready to build and deploy!** 🚀

For questions or issues during deployment, refer to:
- README.md for usage
- INSTALLATION.md for setup
- CHANGELOG.md for version history
