# CoreCycler GUI

A Rust/Iced desktop GUI for [CoreCycler](https://github.com/sp00n/CoreCycler), the CPU stability testing tool.

## What It Does

This GUI wraps the CoreCycler PowerShell script (`script-corecycler.ps1`), providing:

1. **Configuration editor** for CoreCycler INI settings across 9 tabbed sections
2. **Per-test RUN buttons** on each stress test tab (Prime95, yCruncher, yCruncher Old, Aida64, Linpack)
3. **Real-time core status grid** showing which cores are testing/passed/errored
4. **Live log output** from CoreCycler log files with color-coded lines
5. **Process management** using Windows Job Objects to ensure cleanup

## Architecture

```
corecycler-gui/
  src/
    main.rs           - Entry point, window setup (1400x800), forced exit on close
    app.rs            - Main App struct, Message enum (~55 variants), update/view logic
    config/
      mod.rs          - Re-exports
      model.rs        - Typed Rust structs for all 10 INI sections (General, Prime95,
                        Prime95Custom, yCruncher, Aida64, Linpack, AutomaticTestMode,
                        Logging, Update, Debug) with enums for StressTestProgram,
                        Prime95Mode, FFTSize, YCruncherMode (new+old), LinpackVersion, etc.
      parser.rs       - INI file parser and serializer (read/write config.ini),
                        strips inline # comments from values
    runner/
      mod.rs          - Re-exports
      process.rs      - Process spawning with Job Object FFI, config-aware log file
                        monitoring (matches program/mode in log filename), process tree
                        kill via taskkill + named executable fallback for all y-cruncher
                        binaries (new v0.8+ and old v0.7.10), linpack_patched, aida64
    ui/
      mod.rs          - Re-exports
      style.rs        - Dark theme colors (BG_DARK, TEXT_PRIMARY, SUCCESS, ERROR, etc.)
      config_view.rs  - Config editor with 9 tabs, all fields are dropdowns/checkboxes
                        (no text input fields), RUN button on each stress test section
      monitor_view.rs - Right panel: core status grid (8 per row), color-coded log output,
                        COPY button, running program name + elapsed time
```

## Layout

Side-by-side layout (60/40 split):
- **Left panel (60%)**: Config editor with section tabs (General, Prime95, yCruncher, yCruncher Old, Aida64, Linpack, Auto Mode, Logging, Debug) + SAVE/RELOAD/RESET buttons
- **Right panel (40%)**: Monitor showing core status grid + live log output
- **Header**: App title + running program name with elapsed timer + STOP button

## Key Features

### Config Editor (Left Panel)
- Fields use **dropdowns, checkboxes, and text inputs** as appropriate for each setting
- 9 config sections covering CoreCycler's INI structure
- General tab exposes `stressTestProgram` dropdown and `useConfigFile` preset selector
- Stress test sections (Prime95, yCruncher, yCruncher Old, Aida64, Linpack) each have a **RUN** button
- Linpack mode dropdown is locked to FASTEST for versions 2021/2024 (matching script behavior)
- yCruncher new (v0.8+) tests: BKT, BBP, SFTv4, SNT, SVT, FFTv4, N63, VT3
- yCruncher old (v0.7.10) tests: BKT, BBP, SFT, FFT, N32, N64, HNT, VST, C17
- yCruncher old has unique modes: 11-BD1 (Bulldozer), 20-ZN3 (Zen 3)
- AIDA64 mode uses 4 checkboxes (CACHE, CPU, FPU, RAM) matching script's combinable test types
- Cores to Ignore uses multi-checkbox with physical core indices
- Auto Mode section has dropdowns for CO/voltage values from CoreCycler docs
- Unknown/custom values loaded from INI are preserved and shown in dropdowns

### Monitor (Right Panel)
- Core grid: boxes colored by state (Idle/Testing/Passed/Error/Skipped)
- Log output: color-coded (red = "has thrown an error" only, yellow = real WHEA errors only, green = info/success, white = iterations)
- "Checking for stress test errors" and similar diagnostic lines show green, not red
- COPY button copies full log to clipboard
- "Test completed in" and "No core has thrown an error" properly mark cores as passed

### Process Management
- Windows Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`
- `taskkill /PID /T /F` for process tree kill
- Named executable kill as fallback covering all y-cruncher binaries (00-x86 through 24-ZN5, plus old-only 11-BD1 and 20-ZN3), WriteConsoleToWriteFileWrapper, linpack_patched, prime95, prime95_dev, aida64
- `std::process::exit(0)` after iced event loop to kill lingering tokio tasks

### Running Test Flow
1. User clicks RUN on a stress test tab
2. GUI sets `stressTestProgram` in config, clears `useConfigFile`, saves `config.ini`
3. Spawns `powershell.exe -Command "& 'script-corecycler.ps1' *>&1"`
4. Monitors log file matching `{logName}_{date}_{PROGRAM}_{MODE}.log` pattern (config-aware, filters by spawn time)
5. Also watches stdout for log filename hints to resolve exact file faster
6. Parses log lines for core status, iteration count, errors, WHEA
7. Header shows program name + elapsed time; STOP button kills everything

### Known Limitations
- Text input fields accept any script-valid value (custom FFT ranges, per-core CO lists, custom coreTestOrder, etc.) but do not validate syntax
- The GUI and script share one `[yCruncher]` config section; switching between new and old yCruncher may require updating tests
- Two simultaneous GUI launches monitoring the same log directory could see each other's logs

## Dependencies

- `iced` 0.13 (with tokio feature) - GUI framework
- `tokio` (process, io-util, sync, rt-multi-thread, time, fs) - Async runtime
- `sysinfo` 0.33 - Physical core detection

## Building

```bash
cd corecycler-gui
cargo build --release
```

The binary expects to find `script-corecycler.ps1` in the parent directory or current working directory.
