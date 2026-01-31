# Code Modularization Plan

## Current State Analysis

### File Sizes (Lines of Code)
- `main.rs`: **567 lines** ⚠️ **TOO LARGE** - needs refactoring
- `action_executor.rs`: 286 lines - acceptable, could be improved
- `variable_maps.rs`: 254 lines - acceptable (mostly data)
- `key_mapper.rs`: 250 lines - acceptable
- `hid_parser.rs`: 142 lines - ✅ good size

### Current Responsibilities

#### main.rs (567 lines) - **DOING TOO MUCH**
1. Main entry point and CLI argument parsing
2. Logging initialization
3. Mapping file path management
4. Global state management (thread-local storage)
5. **Windows window creation and message loop**
6. **Raw input registration**
7. **Raw input processing**
8. **System tray icon creation and menu handling**
9. **File watching and hot reload**
10. **Configuration reload/reset logic**
11. **Windows service installation/uninstallation**
12. Help text printing
13. Utility functions (widestring conversion)

This violates the **Single Responsibility Principle** - main.rs should orchestrate, not implement!

---

## Proposed Modularization

### Phase 1: Extract Windows-Specific Modules

#### 1.1 Create `src/windows/mod.rs` (Module Directory)
```
src/windows/
├── mod.rs           (declares submodules)
├── window.rs        (window creation and message loop)
├── raw_input.rs     (raw input registration and processing)
└── registry.rs      (install/uninstall service)
```

**Files to create:**

**`src/windows/window.rs`** (~150 lines)
- Functions:
  - `create_window()` → HWND
  - `run_message_loop()` → Result<()>
  - `wnd_proc()` (window procedure callback)
  - Helper: `widestring()` conversion
- Responsibilities: Window creation, message dispatching

**`src/windows/raw_input.rs`** (~100 lines)
- Functions:
  - `register_raw_input(hwnd: HWND)` → Result<()>
  - `handle_raw_input(lparam: LPARAM, mapper: &mut KeyMapper)` → ()
- Responsibilities: HID device registration, raw input processing

**`src/windows/registry.rs`** (~120 lines)
- Functions:
  - `install_service()` → Result<()>
  - `uninstall_service()` → Result<()>
- Responsibilities: Windows startup registry management

**`src/windows/mod.rs`** (~20 lines)
```rust
pub mod window;
pub mod raw_input;
pub mod registry;

pub use window::{create_window, run_message_loop};
pub use raw_input::{register_raw_input, handle_raw_input};
pub use registry::{install_service, uninstall_service};
```

---

#### 1.2 Create `src/tray.rs` (~100 lines)
- Functions:
  - `create_system_tray(exe_dir: &Path)` → Result<TrayIcon>
  - `handle_tray_events()` (runs in thread)
- Responsibilities: System tray icon, menu creation, event handling

---

#### 1.3 Create `src/file_watcher.rs` (~80 lines)
- Functions:
  - `create_file_watcher(path: &Path)` → RecommendedWatcher
  - `handle_file_events(rx: Receiver<()>)` (runs in thread)
- Responsibilities: File system watching, debounced reload triggers

---

### Phase 2: Extract Configuration Management

#### 2.1 Create `src/config.rs` (~120 lines)
- Functions:
  - `get_config_path()` → PathBuf
  - `create_default_mapping_file(path: &Path)` → Result<()>
  - `reload_configuration()`
  - `reset_configuration()`
- Responsibilities: Config file management, default creation
- Would access GLOBAL_MAPPER from thread-local storage

---

### Phase 3: Extract Application State

#### 3.1 Create `src/app_state.rs` (~60 lines)
- Thread-local storage definitions:
  - `GLOBAL_MAPPER`
  - `MAPPING_FILE_PATH`
  - `MAIN_WINDOW`
- Functions:
  - `init_mapper(path: &Path)`
  - `get_mapper()` → Rc<RefCell<KeyMapper>>
  - `send_reload_message()`
  - `send_reset_message()`
  - `send_exit_message()`
- Responsibilities: Centralized global state management

---

### Phase 4: Extract CLI Handling

#### 4.1 Create `src/cli.rs` (~80 lines)
- Functions:
  - `parse_args()` → CliCommand enum
  - `print_help()`
  - `print_version()`
- Enum:
  ```rust
  pub enum CliCommand {
      Run,
      Install,
      Uninstall,
      Help,
      Version,
  }
  ```
- Responsibilities: Command-line argument parsing

---

## New Project Structure

```
src/
├── main.rs                    (~100 lines) ✅ ORCHESTRATION ONLY
├── cli.rs                     (~80 lines)   CLI argument parsing
├── config.rs                  (~120 lines)  Configuration management
├── app_state.rs               (~60 lines)   Global state (thread-local)
├── tray.rs                    (~100 lines)  System tray icon
├── file_watcher.rs            (~80 lines)   File system watching
├── hid_parser.rs              (142 lines)   ✅ NO CHANGE
├── key_mapper.rs              (250 lines)   ✅ NO CHANGE
├── action_executor.rs         (286 lines)   ✅ NO CHANGE
├── variable_maps.rs           (254 lines)   ✅ NO CHANGE
└── windows/
    ├── mod.rs                 (~20 lines)   Module declarations
    ├── window.rs              (~150 lines)  Window & message loop
    ├── raw_input.rs           (~100 lines)  Raw input handling
    └── registry.rs            (~120 lines)  Service install/uninstall
```

**Total LOC: ~1,862 lines** (similar to current, but much better organized)

---

## Simplified main.rs (After Refactoring)

```rust
// --- src/main.rs ---
mod cli;
mod config;
mod app_state;
mod tray;
mod file_watcher;
mod hid_parser;
mod key_mapper;
mod action_executor;
mod variable_maps;
mod windows;

fn main() -> windows::core::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(...)
        .format_timestamp(...)
        .init();

    // Parse CLI arguments
    match cli::parse_args() {
        CliCommand::Install => return windows::registry::install_service(),
        CliCommand::Uninstall => return windows::registry::uninstall_service(),
        CliCommand::Help => { cli::print_help(); return Ok(()); }
        CliCommand::Run => { /* continue */ }
    }

    log::info!("A1314 Daemon v0.2.0 starting...");

    // Initialize configuration
    let config_path = config::get_config_path()?;
    config::ensure_default_exists(&config_path)?;
    
    // Initialize mapper
    app_state::init_mapper(&config_path);
    
    // Create window
    let hwnd = windows::window::create_window()?;
    app_state::store_window_handle(hwnd);
    
    // Register raw input
    windows::raw_input::register_raw_input(hwnd)?;
    
    // Create system tray
    tray::create_system_tray()?;
    
    // Start file watcher
    file_watcher::start_watching(&config_path)?;
    
    // Run message loop (blocks until exit)
    windows::window::run_message_loop()
}
```

---

## Benefits of This Refactoring

### 1. **Single Responsibility Principle**
- Each module has ONE clear purpose
- Easier to understand, test, and maintain

### 2. **Better Testability**
- Windows-specific code isolated → can mock for tests
- Business logic separated from platform code
- Each module can be unit tested independently

### 3. **Cross-Platform Potential**
- Windows-specific code in `src/windows/`
- Could add `src/linux/` or `src/macos/` in the future
- Core logic (parser, mapper, executor) already platform-independent

### 4. **Improved Maintainability**
- ~100-150 lines per file (sweet spot for readability)
- Related functionality grouped together
- Clear module boundaries

### 5. **Better Code Navigation**
- Want to change tray icon? → `src/tray.rs`
- Want to fix registry issues? → `src/windows/registry.rs`
- Want to modify file watching? → `src/file_watcher.rs`

---

## Migration Strategy

### Option A: Big Bang (All at Once)
**Pros:** Clean break, everything organized immediately  
**Cons:** Risky, lots of potential for breakage  
**Time:** 2-3 hours

### Option B: Incremental (Phase by Phase)
**Pros:** Safer, can test after each phase  
**Cons:** Temporary duplication during migration  
**Time:** 4-6 hours (spread over multiple sessions)

**Recommended:** **Option B** - Do one phase at a time:
1. Phase 1: Extract Windows modules (biggest impact)
2. Phase 2: Extract config management
3. Phase 3: Extract app state
4. Phase 4: Extract CLI handling

After each phase:
- Run tests
- Verify compilation
- Test manually

---

## Potential Challenges

### Challenge 1: Thread-Local Storage Access
**Problem:** Multiple modules need access to global state  
**Solution:** Create `app_state` module with helper functions

### Challenge 2: Circular Dependencies
**Problem:** Modules might depend on each other  
**Solution:** Use traits/interfaces, or restructure dependencies

### Challenge 3: Windows API Unsafe Code
**Problem:** Lots of `unsafe` blocks scattered  
**Solution:** Keep unsafe isolated in `windows/` modules with safe wrappers

---

## Files NOT Recommended for Splitting

### ✅ Keep As-Is:
1. **hid_parser.rs** (142 lines) - Perfect size, single responsibility
2. **key_mapper.rs** (250 lines) - Well-structured, cohesive
3. **variable_maps.rs** (254 lines) - Mostly data, splitting doesn't help

### ⚠️ Could Improve (Lower Priority):
**action_executor.rs** (286 lines) - Could split into:
- `src/action_executor/mod.rs` (main logic)
- `src/action_executor/key_sender.rs` (SendInput wrappers)
- `src/action_executor/vk_codes.rs` (parse_key function)
- `src/action_executor/program_launcher.rs` (launch_program, send_app_command)

But this is **optional** - current structure is acceptable.

---

## Questions to Consider

1. **Should we do this refactoring now or wait?**
   - Pros of now: Code is fresh in mind, tests exist
   - Pros of later: Current code works, don't break what isn't broken

2. **Which phase should we tackle first?**
   - Recommend: Phase 1 (Windows modules) - biggest reduction in main.rs

3. **Do we want cross-platform support eventually?**
   - If yes: Definitely extract Windows-specific code
   - If no: Still beneficial for organization

4. **How important is reducing compilation time?**
   - Smaller modules = better incremental compilation
   - Currently all in main.rs = recompile everything on change

---

## Recommendation

**YES, refactor main.rs** - but do it incrementally:

1. **Start with Phase 1** (Windows modules) - extracts ~370 lines from main.rs
2. **Test thoroughly** after Phase 1
3. **Evaluate** - if happy with results, continue with other phases
4. **Keep** other files as-is for now

This gives you:
- Cleaner, more maintainable code
- Better testability
- Foundation for future enhancements
- Minimal risk (incremental approach)

**Estimated Time:** 2-3 hours for Phase 1 alone

---

Would you like me to proceed with implementing any of these phases?
