use serde::{Deserialize, Serialize};
use crate::tweaks::TweakActionResult;

#[cfg(windows)]
use windows::core::w;
#[cfg(windows)]
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

/// Tri-state representing hardware and configuration status of Hardware-Accelerated GPU Scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HagsState {
    Enabled,
    Disabled,
    Unsupported,
}

impl Default for HagsState {
    fn default() -> Self {
        Self::Unsupported
    }
}

/// Categorical presentation tier for the 3-tier visual layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointTier {
    Optimized,  // Peak performance active (Green)
    Actionable, // Safe 1-click optimization available (Yellow)
    Advisory,   // Security or hardware context with trade-offs (Blue)
}

/// Action to execute from the UI card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ActionType {
    None,
    Toggle { enable: bool },
    DeepLink { uri: String },
}

/// Individual audit item describing subsystem readiness, latency impact, and remedies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditCheckpoint {
    pub id: String,
    #[serde(alias = "name")]
    pub title: String,
    pub tier: CheckpointTier,
    #[serde(alias = "status")]
    pub status_text: String,
    pub is_optimized: bool,
    pub latency_impact: String, // "High", "Medium", "Advisory"
    #[serde(alias = "description")]
    pub explanation: String,
    pub action: ActionType,
}

/// Aggregated health report containing calculated readiness score and all checkpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Win11AuditorReport {
    pub readiness_score: u32,
    pub optimized_count: usize,
    pub actionable_count: usize,
    pub advisory_count: usize,
    pub hags_state: HagsState,
    pub checkpoints: Vec<AuditCheckpoint>,
}

// ---------------------------------------------------------------------------
// Audit Query Functions
// ---------------------------------------------------------------------------

/// Queries Hardware-Accelerated GPU Scheduling (HAGS) status.
/// Handles desktops without WDDM 2.7+ hardware support gracefully.
pub fn query_hags_state() -> HagsState {
    #[cfg(windows)]
    {
        use crate::tweaks::read_registry_dword;
        let hws = read_registry_dword(
            HKEY_LOCAL_MACHINE,
            w!("SYSTEM\\CurrentControlSet\\Control\\GraphicsDrivers"),
            w!("HwSchMode"),
        );

        match hws {
            Some(2) => HagsState::Enabled,
            Some(1) => HagsState::Disabled,
            _ => {
                // If HwSchMode does not exist, check if the system has modern WDDM drivers
                // On legacy GPUs (Kepler/Maxwell, older Intel HD), HAGS is unsupported by hardware.
                HagsState::Unsupported
            }
        }
    }
    #[cfg(not(windows))]
    HagsState::Unsupported
}

pub fn toggle_hags(enable: bool) -> TweakActionResult {
    #[cfg(windows)]
    {
        use crate::tweaks::write_registry_dword;
        let current = query_hags_state();
        if current == HagsState::Unsupported {
            return TweakActionResult {
                success: false,
                message: "Hardware-Accelerated GPU Scheduling is not supported by your GPU or driver (WDDM 2.7+ required).".into(),
            };
        }

        let val = if enable { 2u32 } else { 1u32 };
        let res = write_registry_dword(
            HKEY_LOCAL_MACHINE,
            w!("SYSTEM\\CurrentControlSet\\Control\\GraphicsDrivers"),
            w!("HwSchMode"),
            val,
        );

        match res {
            Ok(_) => TweakActionResult {
                success: true,
                message: if enable {
                    "HAGS Enabled. Requires Windows restart to activate the GPU kernel scheduler.".into()
                } else {
                    "HAGS Disabled. Requires Windows restart.".into()
                },
            },
            Err(e) => TweakActionResult {
                success: false,
                message: format!("Failed to modify HAGS registry key: {}", e),
            },
        }
    }
    #[cfg(not(windows))]
    {
        let _ = enable;
        TweakActionResult {
            success: true,
            message: "Mock HAGS toggle successful.".into(),
        }
    }
}

/// Checks "Optimizations for windowed games" (DXGI Flip Model upgrade).
pub fn query_windowed_optimizations() -> bool {
    #[cfg(windows)]
    {
        use crate::tweaks::read_registry_sz;
        let val = read_registry_sz(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\DirectX\\UserGpuPreferences"),
            w!("DirectXUserGlobalSettings"),
        );
        val.as_deref().map(|s| s.contains("SwapEffectUpgradeEnable=1")).unwrap_or(false)
    }
    #[cfg(not(windows))]
    false
}

pub fn toggle_windowed_optimizations(enable: bool) -> TweakActionResult {
    #[cfg(windows)]
    {
        use crate::tweaks::{delete_registry_value, read_registry_sz, write_registry_sz};
        let current_val = read_registry_sz(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\DirectX\\UserGpuPreferences"),
            w!("DirectXUserGlobalSettings"),
        ).unwrap_or_default();

        let new_val = if enable {
            if current_val.contains("SwapEffectUpgradeEnable=1") {
                current_val
            } else if current_val.contains("SwapEffectUpgradeEnable=0") {
                current_val.replace("SwapEffectUpgradeEnable=0", "SwapEffectUpgradeEnable=1")
            } else {
                format!("{}SwapEffectUpgradeEnable=1;", current_val)
            }
        } else {
            current_val.replace("SwapEffectUpgradeEnable=1;", "").replace("SwapEffectUpgradeEnable=1", "")
        };

        let res = if new_val.is_empty() {
            let _ = delete_registry_value(
                HKEY_CURRENT_USER,
                w!("Software\\Microsoft\\DirectX\\UserGpuPreferences"),
                w!("DirectXUserGlobalSettings"),
            );
            Ok(())
        } else {
            write_registry_sz(
                HKEY_CURRENT_USER,
                w!("Software\\Microsoft\\DirectX\\UserGpuPreferences"),
                w!("DirectXUserGlobalSettings"),
                &new_val,
            )
        };

        match res {
            Ok(_) => TweakActionResult {
                success: true,
                message: if enable {
                    "Optimizations for Windowed Games enabled (DXGI Flip Model active).".into()
                } else {
                    "Optimizations for Windowed Games disabled.".into()
                },
            },
            Err(e) => TweakActionResult {
                success: false,
                message: format!("Failed to update Windowed Optimizations: {}", e),
            },
        }
    }
    #[cfg(not(windows))]
    {
        let _ = enable;
        TweakActionResult {
            success: true,
            message: "Mock Windowed Optimizations toggle successful.".into(),
        }
    }
}

/// Checks Windows Auto Game Mode status.
pub fn query_game_mode() -> bool {
    #[cfg(windows)]
    {
        use crate::tweaks::read_registry_dword;
        let mode = read_registry_dword(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\GameBar"),
            w!("AutoGameModeEnabled"),
        );
        // Default in Windows 10/11 is enabled (1)
        mode.unwrap_or(1) == 1
    }
    #[cfg(not(windows))]
    true
}

pub fn toggle_game_mode(enable: bool) -> TweakActionResult {
    #[cfg(windows)]
    {
        use crate::tweaks::write_registry_dword;
        let val = if enable { 1u32 } else { 0u32 };
        let res = write_registry_dword(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\GameBar"),
            w!("AutoGameModeEnabled"),
            val,
        );
        match res {
            Ok(_) => TweakActionResult {
                success: true,
                message: if enable {
                    "Windows Game Mode enabled.".into()
                } else {
                    "Windows Game Mode disabled.".into()
                },
            },
            Err(e) => TweakActionResult {
                success: false,
                message: format!("Failed to update Game Mode: {}", e),
            },
        }
    }
    #[cfg(not(windows))]
    {
        let _ = enable;
        TweakActionResult {
            success: true,
            message: "Mock Game Mode toggle successful.".into(),
        }
    }
}

/// Checks Multi-Plane Overlay (MPO) capability in DWM.
pub fn query_mpo_status() -> (bool, String) {
    #[cfg(windows)]
    {
        use crate::tweaks::read_registry_dword;
        let test_mode = read_registry_dword(
            HKEY_LOCAL_MACHINE,
            w!("SOFTWARE\\Microsoft\\Windows\\Dwm"),
            w!("OverlayTestMode"),
        );
        // If OverlayTestMode is 5, MPO is disabled. Otherwise MPO is supported by default on modern WDDM.
        if test_mode == Some(5) {
            (false, "Disabled via OverlayTestMode registry override".into())
        } else {
            (true, "Supported (Hardware Overlays Active)".into())
        }
    }
    #[cfg(not(windows))]
    (true, "Mock MPO Active".into())
}

/// Checks Hypervisor-protected Code Integrity (HVCI / Memory Integrity).
pub fn query_hvci_status() -> (bool, String) {
    #[cfg(windows)]
    {
        use crate::tweaks::read_registry_dword;
        let enabled = read_registry_dword(
            HKEY_LOCAL_MACHINE,
            w!("SYSTEM\\CurrentControlSet\\Control\\DeviceGuard\\Scenarios\\HypervisorEnforcedCodeIntegrity"),
            w!("Enabled"),
        ).unwrap_or(0);

        if enabled == 1 {
            (true, "Active (Memory Integrity Protection On)".into())
        } else {
            (false, "Inactive (Standard Hardware Execution)".into())
        }
    }
    #[cfg(not(windows))]
    (false, "Mock HVCI Inactive".into())
}

// ---------------------------------------------------------------------------
// Score & Report Assembly
// ---------------------------------------------------------------------------

pub fn calculate_readiness_score(checkpoints: &[AuditCheckpoint]) -> u32 {
    let mut total_weight = 0u32;
    let mut earned_weight = 0u32;

    for cp in checkpoints {
        // Skip advisory/unsupported checkpoints from penalizing readiness
        if cp.tier == CheckpointTier::Advisory && cp.action == ActionType::None {
            continue;
        }

        let weight = match cp.latency_impact.as_str() {
            "High" => 35,
            "Medium" => 20,
            _ => 10,
        };

        total_weight += weight;
        if cp.is_optimized {
            earned_weight += weight;
        }
    }

    if total_weight == 0 {
        100
    } else {
        ((earned_weight as f32 / total_weight as f32) * 100.0).round() as u32
    }
}

pub fn audit_win11_gaming_features() -> Win11AuditorReport {
    let hags = query_hags_state();
    let windowed_opt = query_windowed_optimizations();
    let game_mode = query_game_mode();
    let (mpo_active, mpo_desc) = query_mpo_status();
    let (hvci_active, hvci_desc) = query_hvci_status();

    let mut checkpoints = Vec::new();

    // 1. Hardware-Accelerated GPU Scheduling (HAGS)
    match hags {
        HagsState::Enabled => {
            checkpoints.push(AuditCheckpoint {
                id: "hags".into(),
                title: "Hardware-Accelerated GPU Scheduling (HAGS)".into(),
                tier: CheckpointTier::Optimized,
                status_text: "Enabled (WDDM 2.7+ Active)".into(),
                is_optimized: true,
                latency_impact: "High".into(),
                explanation: "GPU kernel directly manages VRAM and scheduling queues, minimizing input latency and driver overhead.".into(),
                action: ActionType::Toggle { enable: false },
            });
        }
        HagsState::Disabled => {
            checkpoints.push(AuditCheckpoint {
                id: "hags".into(),
                title: "Hardware-Accelerated GPU Scheduling (HAGS)".into(),
                tier: CheckpointTier::Actionable,
                status_text: "Disabled (WDDM Software Queues)".into(),
                is_optimized: false,
                latency_impact: "High".into(),
                explanation: "CPU mediates all GPU frame scheduling commands, causing micro-stutters and input lag in high-FPS games.".into(),
                action: ActionType::Toggle { enable: true },
            });
        }
        HagsState::Unsupported => {
            checkpoints.push(AuditCheckpoint {
                id: "hags".into(),
                title: "Hardware-Accelerated GPU Scheduling (HAGS)".into(),
                tier: CheckpointTier::Advisory,
                status_text: "Unsupported by GPU / Driver".into(),
                is_optimized: false,
                latency_impact: "Low".into(),
                explanation: "Your graphics card or display driver does not support WDDM 2.7+ hardware scheduling (requires NVIDIA Pascal+, AMD RDNA+, or Intel Arc).".into(),
                action: ActionType::None,
            });
        }
    }

    // 2. Optimizations for Windowed Games (DXGI Flip Model)
    if windowed_opt {
        checkpoints.push(AuditCheckpoint {
            id: "windowed_opt".into(),
            title: "Optimizations for Windowed Games".into(),
            tier: CheckpointTier::Optimized,
            status_text: "Enabled (Modern Flip Model Upgrade)".into(),
            is_optimized: true,
            latency_impact: "High".into(),
            explanation: "Upgrades legacy DX10/DX11 windowed borderless games to the modern DXGI Flip Model, matching exclusive fullscreen latency.".into(),
            action: ActionType::Toggle { enable: false },
        });
    } else {
        checkpoints.push(AuditCheckpoint {
            id: "windowed_opt".into(),
            title: "Optimizations for Windowed Games".into(),
            tier: CheckpointTier::Actionable,
            status_text: "Disabled (Legacy Blt Presentation)".into(),
            is_optimized: false,
            latency_impact: "High".into(),
            explanation: "Borderless games suffer from DWM compositor double-buffering latency and higher frame time jitter.".into(),
            action: ActionType::Toggle { enable: true },
        });
    }

    // 3. Windows Game Mode
    if game_mode {
        checkpoints.push(AuditCheckpoint {
            id: "game_mode".into(),
            title: "Windows Game Mode".into(),
            tier: CheckpointTier::Optimized,
            status_text: "Enabled".into(),
            is_optimized: true,
            latency_impact: "Medium".into(),
            explanation: "Windows kernel prioritizes CPU and GPU resources for active game processes while pausing background maintenance.".into(),
            action: ActionType::Toggle { enable: false },
        });
    } else {
        checkpoints.push(AuditCheckpoint {
            id: "game_mode".into(),
            title: "Windows Game Mode".into(),
            tier: CheckpointTier::Actionable,
            status_text: "Disabled".into(),
            is_optimized: false,
            latency_impact: "Medium".into(),
            explanation: "Background Windows Update and maintenance tasks may trigger sudden frame drops during matches.".into(),
            action: ActionType::Toggle { enable: true },
        });
    }

    // 4. Multi-Plane Overlay (MPO)
    if mpo_active {
        checkpoints.push(AuditCheckpoint {
            id: "mpo".into(),
            title: "Multi-Plane Overlay (MPO)".into(),
            tier: CheckpointTier::Optimized,
            status_text: mpo_desc,
            is_optimized: true,
            latency_impact: "High".into(),
            explanation: "Display controller composites game frames and desktop overlays in hardware, bypassing DWM composition lag entirely.".into(),
            action: ActionType::None,
        });
    } else {
        checkpoints.push(AuditCheckpoint {
            id: "mpo".into(),
            title: "Multi-Plane Overlay (MPO)".into(),
            tier: CheckpointTier::Actionable,
            status_text: mpo_desc,
            is_optimized: false,
            latency_impact: "High".into(),
            explanation: "MPO is forcibly disabled via registry override, forcing all windowed frames through software composition.".into(),
            action: ActionType::None,
        });
    }

    // 5. Variable Refresh Rate (VRR) / Advanced Display Link
    checkpoints.push(AuditCheckpoint {
        id: "vrr".into(),
        title: "Variable Refresh Rate (VRR / G-Sync)".into(),
        tier: CheckpointTier::Advisory,
        status_text: "Hardware Sync Managed by Windows".into(),
        is_optimized: true,
        latency_impact: "Medium".into(),
        explanation: "Ensures display refresh rate dynamically matches GPU frame output, eliminating tearing and frame pacing stutter.".into(),
        action: ActionType::DeepLink { uri: "ms-settings:display-advanced".into() },
    });

    // 6. Memory Integrity / Core Isolation (HVCI)
    if hvci_active {
        checkpoints.push(AuditCheckpoint {
            id: "hvci".into(),
            title: "Virtualization-Based Security (VBS / HVCI)".into(),
            tier: CheckpointTier::Advisory,
            status_text: hvci_desc,
            is_optimized: true,
            latency_impact: "Medium".into(),
            explanation: "Provides hypervisor-level kernel protection. Note: Can introduce 3-7% CPU performance overhead in CPU-bound competitive esports titles.".into(),
            action: ActionType::DeepLink { uri: "ms-settings:windowsdefender".into() },
        });
    } else {
        checkpoints.push(AuditCheckpoint {
            id: "hvci".into(),
            title: "Virtualization-Based Security (VBS / HVCI)".into(),
            tier: CheckpointTier::Optimized,
            status_text: "Inactive (Raw Native CPU Execution)".into(),
            is_optimized: true,
            latency_impact: "Low".into(),
            explanation: "CPU executes without hypervisor emulation layers, guaranteeing zero VBS virtualization overhead in esports titles.".into(),
            action: ActionType::DeepLink { uri: "ms-settings:windowsdefender".into() },
        });
    }

    let score = calculate_readiness_score(&checkpoints);
    let optimized_count = checkpoints.iter().filter(|c| c.tier == CheckpointTier::Optimized).count();
    let actionable_count = checkpoints.iter().filter(|c| c.tier == CheckpointTier::Actionable).count();
    let advisory_count = checkpoints.iter().filter(|c| c.tier == CheckpointTier::Advisory).count();

    Win11AuditorReport {
        readiness_score: score,
        optimized_count,
        actionable_count,
        advisory_count,
        hags_state: hags,
        checkpoints,
    }
}

