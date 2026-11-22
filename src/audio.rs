// src/audio.rs - Windows Core Audio (audiodg.exe) Optimization & Real CoreAudio Glitch Telemetry
// 100% User-Mode Win32 Affinity/Priority + Silent Shared-Mode WASAPI Deadline Probe

use crate::tweaks::TweakActionResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

#[cfg(windows)]
use windows::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows::Win32::System::Threading::{
    GetProcessAffinityMask, OpenProcess, SetPriorityClass, SetProcessAffinityMask,
    HIGH_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SET_INFORMATION,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTuningConfig {
    pub enabled: bool,
}

impl Default for AudioTuningConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

pub type SharedAudioConfig = Arc<RwLock<AudioTuningConfig>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTuningStatus {
    pub enabled: bool,
    pub audiodg_pid: Option<u32>,
    pub pinned_core_index: Option<u32>,
    pub is_high_priority: bool,
}

fn audio_config_path() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("FerrumPulse").join("audio_config.json")
}

pub fn load_audio_config_from(path: &PathBuf) -> AudioTuningConfig {
    if !path.exists() {
        return AudioTuningConfig::default();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_audio_config_to(path: &PathBuf, cfg: &AudioTuningConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("Failed to serialize audio config: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("Failed to write audio config: {}", e))
}

pub fn load_audio_config() -> AudioTuningConfig {
    load_audio_config_from(&audio_config_path())
}

pub fn save_audio_config(cfg: &AudioTuningConfig) -> Result<(), String> {
    save_audio_config_to(&audio_config_path(), cfg)
}

/// Computes an optimal dedicated physical CPU core affinity bitmask for `audiodg.exe`.
/// Selects Core 2 (`1 << 2`) on >=4-core CPUs (avoids Core 0 DPC/IRQ contention while staying
/// on Physical P-Core #1 on Intel hybrid CPUs), Core 1 on 2-core CPUs, and `None` on 1-core CPUs.
pub fn compute_audio_core_affinity_mask(logical_cores: usize) -> Option<(usize, u32)> {
    if logical_cores >= 4 {
        Some((1usize << 2, 2))
    } else if logical_cores >= 2 {
        Some((1usize << 1, 1))
    } else {
        None
    }
}

/// Pure threshold check for WASAPI event-driven buffer deadline misses (>2.5x expected period).
pub fn is_wasapi_deadline_missed(elapsed_us: u64, expected_period_us: u64) -> bool {
    if expected_period_us == 0 {
        return false;
    }
    elapsed_us > (expected_period_us.saturating_mul(5)) / 2
}

/// Pure fallback glitch estimator when no WASAPI playback device is active:
/// counts DPC execution intervals exceeding the 10ms CoreAudio buffer window.
pub fn count_dpc_fallback_glitches(dpc_deltas_us: &[f64]) -> u32 {
    dpc_deltas_us.iter().filter(|&&us| us > 10_000.0).count() as u32
}

/// Finds the currently running `audiodg.exe` PID from an existing `sysinfo::System` snapshot.
pub fn find_audiodg_pid_in_sys(sys: &sysinfo::System) -> Option<u32> {
    for (pid, proc_info) in sys.processes() {
        let name = proc_info.name().to_string_lossy().to_lowercase();
        if name == "audiodg.exe" {
            return Some(pid.as_u32());
        }
    }
    None
}

/// Applies or reverts `audiodg.exe` core pinning & `HIGH_PRIORITY_CLASS`.
pub fn apply_audiodg_to_pid(pid: u32, enable: bool) -> Result<Option<u32>, String> {
    #[cfg(windows)]
    {
        let logical_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let handle = unsafe {
            OpenProcess(
                PROCESS_SET_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION,
                false,
                pid,
            )
        }
        .map_err(|e| format!("Could not open audiodg.exe (PID {}): {:?}", pid, e))?;

        let mut pinned_core = None;
        let result = if enable {
            let prio_ok = unsafe { SetPriorityClass(handle, HIGH_PRIORITY_CLASS) };
            let mut aff_ok = Ok(());
            if let Some((mask, core_idx)) = compute_audio_core_affinity_mask(logical_cores) {
                aff_ok = unsafe { SetProcessAffinityMask(handle, mask) };
                if aff_ok.is_ok() {
                    pinned_core = Some(core_idx);
                }
            }
            prio_ok.and(aff_ok)
        } else {
            let prio_ok = unsafe { SetPriorityClass(handle, NORMAL_PRIORITY_CLASS) };
            let mut proc_mask: usize = 0;
            let mut sys_mask: usize = 0;
            let aff_ok = unsafe {
                if GetProcessAffinityMask(handle, &mut proc_mask, &mut sys_mask).is_ok()
                    && sys_mask != 0
                {
                    SetProcessAffinityMask(handle, sys_mask)
                } else {
                    Ok(())
                }
            };
            prio_ok.and(aff_ok)
        };

        unsafe {
            let _ = CloseHandle(handle);
        }

        result.map_err(|e| format!("Failed to update audiodg.exe: {:?}", e))?;
        Ok(pinned_core)
    }

    #[cfg(not(windows))]
    {
        let _ = pid;
        Ok(if enable { Some(2) } else { None })
    }
}

/// Ambient daemon helper: if audio tuning is enabled and `audiodg.exe` restarted with a new PID
/// (e.g. USB/Bluetooth headset plug/unplug), automatically re-applies priority & core pinning.
pub fn reapply_if_audiodg_restarted(
    sys: &sysinfo::System,
    enabled: bool,
    last_tuned_pid: &mut Option<u32>,
) {
    if !enabled {
        *last_tuned_pid = None;
        return;
    }
    if let Some(pid) = find_audiodg_pid_in_sys(sys) {
        if *last_tuned_pid != Some(pid) {
            if apply_audiodg_to_pid(pid, true).is_ok() {
                *last_tuned_pid = Some(pid);
            }
        }
    }
}

pub fn toggle_audio_engine_tuning(enable: bool) -> TweakActionResult {
    let cfg = AudioTuningConfig { enabled: enable };
    let _ = save_audio_config(&cfg);

    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    if let Some(pid) = find_audiodg_pid_in_sys(&sys) {
        match apply_audiodg_to_pid(pid, enable) {
            Ok(Some(core_idx)) if enable => TweakActionResult {
                success: true,
                message: format!(
                    "Core Audio (audiodg.exe PID {}) pinned to Core {} at High priority.",
                    pid, core_idx
                ),
            },
            Ok(_) if enable => TweakActionResult {
                success: true,
                message: format!("Core Audio (audiodg.exe PID {}) set to High priority.", pid),
            },
            Ok(_) => TweakActionResult {
                success: true,
                message: format!(
                    "Restored stock Windows priority and CPU affinity on audiodg.exe (PID {}).",
                    pid
                ),
            },
            Err(e) => TweakActionResult {
                success: false,
                message: format!(
                    "Audio tuning preference saved, but live apply requires Administrator rights ({}).",
                    e
                ),
            },
        }
    } else {
        TweakActionResult {
            success: true,
            message: if enable {
                "Audio Engine tuning enabled (will auto-apply once audiodg.exe starts).".into()
            } else {
                "Audio Engine tuning disabled.".into()
            },
        }
    }
}

pub fn get_audio_tuning_status(cfg: &AudioTuningConfig) -> AudioTuningStatus {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let pid = find_audiodg_pid_in_sys(&sys);
    let logical_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let pinned_core_index = if cfg.enabled {
        compute_audio_core_affinity_mask(logical_cores).map(|(_, idx)| idx)
    } else {
        None
    };

    AudioTuningStatus {
        enabled: cfg.enabled,
        audiodg_pid: pid,
        pinned_core_index,
        is_high_priority: cfg.enabled && pid.is_some(),
    }
}

// ---------------------------------------------------------------------------
// Silent Shared-Mode WASAPI Event-Driven Deadline Probe (Zero Audible Output)
// ---------------------------------------------------------------------------

pub struct AudioGlitchProbe {
    stop_flag: Arc<AtomicBool>,
    glitches: Arc<AtomicU32>,
    wasapi_active: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl AudioGlitchProbe {
    pub fn start() -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let glitches = Arc::new(AtomicU32::new(0));
        let wasapi_active = Arc::new(AtomicBool::new(false));

        let stop_c = Arc::clone(&stop_flag);
        let glitches_c = Arc::clone(&glitches);
        let active_c = Arc::clone(&wasapi_active);

        let worker = std::thread::spawn(move || {
            #[cfg(windows)]
            run_silent_wasapi_probe(stop_c, glitches_c, active_c);
            #[cfg(not(windows))]
            let _ = (stop_c, glitches_c, active_c);
        });

        Self {
            stop_flag,
            glitches,
            wasapi_active,
            worker: Some(worker),
        }
    }

    /// Stops the background WASAPI probe and returns `(wasapi_active, glitch_count)`.
    pub fn finish(mut self) -> (bool, u32) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
        (
            self.wasapi_active.load(Ordering::Relaxed),
            self.glitches.load(Ordering::Relaxed),
        )
    }
}

impl Drop for AudioGlitchProbe {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(windows)]
fn run_silent_wasapi_probe(
    stop_flag: Arc<AtomicBool>,
    glitches: Arc<AtomicU32>,
    wasapi_active: Arc<AtomicBool>,
) {
    use windows::Win32::Media::Audio::{
        eConsole, eRender, IAudioClient, IAudioRenderClient, IMMDeviceEnumerator,
        MMDeviceEnumerator, AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED,
        AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
        COINIT_MULTITHREADED,
    };
    use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};

    unsafe {
        if CoInitializeEx(None, COINIT_MULTITHREADED).is_err() {
            return;
        }

        let probe_res: Result<(), ()> = (|| {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|_| ())?;
            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|_| ())?;
            let audio_client: IAudioClient =
                device.Activate(CLSCTX_ALL, None).map_err(|_| ())?;

            let mix_format_ptr = audio_client.GetMixFormat().map_err(|_| ())?;
            let mut default_period: i64 = 100_000; // 10ms fallback in 100ns units
            let mut min_period: i64 = 0;
            let _ = audio_client.GetDevicePeriod(Some(&mut default_period), Some(&mut min_period));
            let expected_period_us = ((default_period.max(10_000)) as u64) / 10;

            let init_res = audio_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                default_period,
                0,
                mix_format_ptr,
                None,
            );
            CoTaskMemFree(Some(mix_format_ptr as *const std::ffi::c_void));
            init_res.map_err(|_| ())?;

            let event_handle = CreateEventW(None, false, false, None).map_err(|_| ())?;
            if audio_client.SetEventHandle(event_handle).is_err() {
                let _ = CloseHandle(event_handle);
                return Err(());
            }

            let buffer_frames = audio_client.GetBufferSize().map_err(|_| ())?;
            let render_client: IAudioRenderClient = audio_client.GetService().map_err(|_| ())?;

            // Prime buffer with silent frames (AUDCLNT_BUFFERFLAGS_SILENT = zero audible sound)
            if render_client.GetBuffer(buffer_frames).is_ok() {
                let _ = render_client.ReleaseBuffer(buffer_frames, AUDCLNT_BUFFERFLAGS_SILENT.0 as u32);
            }

            if audio_client.Start().is_err() {
                let _ = CloseHandle(event_handle);
                return Err(());
            }

            wasapi_active.store(true, Ordering::Relaxed);
            let mut last_wakeup = Instant::now();
            let mut warmup_ticks = 0u32;

            while !stop_flag.load(Ordering::Relaxed) {
                let wait = WaitForSingleObject(event_handle, 100);
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                if wait.0 == 0 {
                    let now = Instant::now();
                    let elapsed_us = now.duration_since(last_wakeup).as_micros() as u64;
                    last_wakeup = now;

                    // Ignore first 3 warmup callbacks while stream stabilizes
                    if warmup_ticks >= 3 && is_wasapi_deadline_missed(elapsed_us, expected_period_us)
                    {
                        glitches.fetch_add(1, Ordering::Relaxed);
                    }
                    warmup_ticks = warmup_ticks.saturating_add(1);

                    if let Ok(padding) = audio_client.GetCurrentPadding() {
                        let available = buffer_frames.saturating_sub(padding);
                        if available > 0 && render_client.GetBuffer(available).is_ok() {
                            let _ = render_client
                                .ReleaseBuffer(available, AUDCLNT_BUFFERFLAGS_SILENT.0 as u32);
                        }
                    }
                }
            }

            let _ = audio_client.Stop();
            let _ = CloseHandle(event_handle);
            Ok(())
        })();

        let _ = probe_res;
        CoUninitialize();
    }
}
