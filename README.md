# CoreCycler GUI

A vibecoded Rust/Iced GUI wrapper for [sp00n's CoreCycler](https://github.com/sp00n/CoreCycler), by **FPSHEAVEN**.

CoreCycler is a CPU stability testing tool for PBO & Curve Optimizer on AMD Ryzen (or any overclocking/undervolting scenario). This GUI replaces the manual INI editing workflow with a visual config editor and real-time test monitoring.

## Features

- **9-tab config editor** covering all CoreCycler INI settings (General, Prime95, yCruncher, yCruncher Old, Aida64, Linpack, Auto Mode, Logging, Debug)
- **Per-test RUN buttons** on each stress test tab — click and go
- **Real-time core status grid** showing Idle / Testing / Passed / Error / Skipped per core
- **Live color-coded log output** parsed from CoreCycler log files
- **Process management** with Windows Job Objects for clean start/stop

## Screenshot

*(coming soon)*

## Quick Start (Download)

1. Download the latest **CoreCycler-GUI.exe** from [Releases](https://github.com/fpsheaven/CoreCycler/releases)
2. Place it in your CoreCycler root directory (next to `script-corecycler.ps1`)
3. Run it — no Rust or extra runtime needed, it's a self-contained executable

> Don't have CoreCycler yet? Download it from [sp00n's repo](https://github.com/sp00n/CoreCycler), or just clone this fork which includes both the script and the GUI.

## Building From Source

If you want to build it yourself:

1. Install [Rust](https://rustup.rs/)
2. Build:
   ```bash
   cd corecycler-gui
   cargo build --release
   ```
3. The binary will be at `corecycler-gui/target/release/corecycler-gui.exe`

## How It Works

The GUI reads and writes CoreCycler's `config.ini`, then spawns `script-corecycler.ps1` via PowerShell when you click RUN. It monitors the log files in real-time to display core status and test progress. When you click STOP, it kills the entire process tree cleanly.

## Supported Stress Tests

| Program | Versions / Modes |
|---------|-----------------|
| **Prime95** | SSE, AVX, AVX2, AVX512, Custom |
| **yCruncher (new v0.8+)** | BKT, BBP, SFTv4, SNT, SVT, FFTv4, N63, VT3 |
| **yCruncher (old v0.7.10)** | BKT, BBP, SFT, FFT, N32, N64, HNT, VST, C17 |
| **Aida64** | CACHE, CPU, FPU, RAM (combinable) |
| **Linpack** | 2021, 2024 (mode locked to FASTEST), 2019 |

## Credits

- **[sp00n](https://github.com/sp00n)** — Creator of [CoreCycler](https://github.com/sp00n/CoreCycler), the PowerShell script that does all the actual testing
- **FPSHEAVEN** — This GUI wrapper
- **Claude (Anthropic)** — Vibecode co-pilot

## License

This fork inherits CoreCycler's original license. See [LICENSE](LICENSE) for details.
