# FerrumPulse ⚡
> Lightweight, Transparent Windows Gaming & PC Optimizer written in Rust with Tauri v2

FerrumPulse is a safe, open-source alternative to black-box "TikTok optimizer .exe" scripts. It manipulates official Win32 power management, process scheduling, and working-set memory APIs directly without disabling core Windows features or triggering anti-cheat flags.

---

## 🛡️ Anti-Cheat & System Safety Contract
Unlike destructive optimizer tools:
* **Anti-Cheat Safe**: No DLL injection, memory scanning, code hooking, or unsigned kernel drivers. Vanguard, Easy Anti-Cheat (EAC), and BattlEye remain fully operational.
* **Non-Destructive**: Never disables Windows Defender, Windows Update, Microsoft Store, Print Spooler, or Windows Audio.
* **100% Revertible**: Every tweak preserves safe Windows defaults with one-click rollback.

---

## 🚀 Core Modules & Win32 APIs

### 1. CPU & Power Management (`src/power.rs`)
* Target: Win32 `PowrProf` (`54533251-82be-4824-96c1-47b60b740d00` processor settings subgroup).
* Core Unparking: Directly toggles AC Min Cores index via `0cc5b647-c1df-4637-891a-dec35c318583` (`PowerWriteACValueIndex`) between **100% (Unparked)** and **50% (Stock)** to eliminate micro-stutters from CPU core sleep states.
* Scheme Switcher: Instant activation of **High Performance** (`8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c`), **Ultimate Performance** (`e9a42b02-d5df-448d-aa00-03f14749eb61`), or **Balanced** (`381b4222-f694-41f0-9685-ff5bb260df2e`).

### 2. Dynamic Process Pacing & Game Priority Daemon (`src/process.rs`)
* Ambient 1000ms polling daemon monitoring active foreground focus via `GetForegroundWindow` and `GetWindowThreadProcessId`.
* Automatically promotes recognized active games (`cs2.exe`, `valorant.exe`, `cyberpunk2077.exe`, etc.) to `HIGH_PRIORITY_CLASS`.
* Applies `ProcessPowerThrottling` (`PROCESS_POWER_THROTTLING_EXECUTION_SPEED` / EcoQoS) to background launchers and browsers (Discord, Chrome, Steam, Battle.net) while gaming.
* Restores normal priority classes and removes throttling automatically when Alt-Tabbing or exiting games.

### 3. RAM Working-Set Trimmer (`src/memory.rs`)
* Flushes dormant, unreferenced pages from active processes back into the paging pool via `K32EmptyWorkingSet`.
* Safely skips system/kernel PIDs (`<= 4`) and protected processes to prevent instability.
* Live before/after measurement with `GlobalMemoryStatusEx`.

### 4. Safe Windows Debloat & Network Pacing (`src/tweaks.rs`)
* **Telemetry**: Stops and disables `DiagTrack` (Connected User Experiences and Telemetry) service using Windows SCM (`OpenSCManagerW`, `ChangeServiceConfigW`). Reverts to Manual start.
* **Gaming Overlays**: Toggles Windows Game Bar DVR (`AppCaptureEnabled` and `GameDVR_Enabled` in `HKCU\System\GameConfigStore`) to avoid background video buffer overhead.
* **Network & Multimedia Pacing**: Sets `NetworkThrottlingIndex = 0xffffffff` (disabling network throttling) and `SystemResponsiveness = 0` (100% CPU priority to games) in `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile`.

### 5. Storage & DirectX Shader Cache Cleaner (`src/cleaner.rs`)
* Scans and purges:
  - User Temp (`%TEMP%`)
  - Windows Temp (`C:\Windows\Temp`)
  - DirectX Shader Caches (`%LOCALAPPDATA%\D3DSCache`, NVIDIA & AMD `DxCache`)
  - Windows Recycle Bin (`SHEmptyRecycleBinW`)
* Handles locked files gracefully by skipping them without interrupting the cleanup routine.

---

## 🛠️ Project Structure
```text
FerrumPulse/
├── Cargo.toml               # Rust dependencies & Windows API feature flags
├── build.rs                 # WinRes build script compiling app.manifest
├── app.manifest             # UAC Administrator elevation & PerMonitorV2 DPI
├── tauri.conf.json          # Tauri v2 configuration (tray, window flags)
├── src/
│   ├── main.rs              # App entrypoint (windows_subsystem = "windows")
│   ├── lib.rs               # Tauri IPC commands & application lifecycle
│   ├── power.rs             # PowrProf core unparking & power schemes
│   ├── process.rs           # Dynamic game priority & EcoQoS daemon
│   ├── memory.rs            # Working-set RAM purge via K32EmptyWorkingSet
│   ├── tweaks.rs            # DiagTrack, GameDVR & Network Pacing
│   ├── cleaner.rs           # Storage & DirectX shader cache cleaner
│   └── tray.rs              # System tray icon, menu & minimize-to-tray
├── tests/
│   └── smoke_test.rs        # Test harness for Win32 modules & configs
└── ui/                      # Dark-Mode Bento UI
    ├── index.html           # Modular Bento-card dashboard layout
    ├── styles/
    │   ├── main.css         # Dark theme (Nord/OLED, glassmorphism)
    │   └── bento.css        # Bento grid styles & gauges
    └── src/
        └── app.js           # Frontend IPC bridge & live polling loop
```

---

## 🔨 Building & Running

### Requirements
* Windows 10 / 11 (64-bit)
* Rust toolchain (1.80+) with MSVC C++ Build Tools
* Administrator privileges (enforced via `app.manifest`)

### Run in Development Mode
```powershell
# Run smoke tests
cargo test --test smoke_test

# Launch the desktop utility
cargo run
```

### Build Production Binary
```powershell
cargo build --release
```
The output binary will be located at `target/release/ferrumpulse.exe`. When launched, Windows UAC will prompt for elevation, and the app will dock into the system tray when closed.

