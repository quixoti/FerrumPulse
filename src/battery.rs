use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[cfg(windows)]
use windows::core::GUID;
#[cfg(windows)]
use windows::Win32::System::Power::{
    GetSystemPowerStatus, PowerSetActiveScheme, PowerWriteDCValueIndex,
    SYSTEM_POWER_STATUS,
};
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::{
    ChangeDisplaySettingsExW, EnumDisplaySettingsW, CDS_UPDATEREGISTRY, DEVMODEW,
    DISP_CHANGE_SUCCESSFUL, ENUM_CURRENT_SETTINGS,
};

/// Power Saver Scheme GUID: a1841308-3540-4fab-bc81-f71556f20b4a
#[cfg(windows)]
pub const GUID_POWER_SAVER: GUID = GUID {
    data1: 0xa1841308,
    data2: 0x3540,
    data3: 0x4fab,
    data4: [0xbc, 0x81, 0xf7, 0x15, 0x56, 0xf2, 0x0b, 0x4a],
};

/// Balanced Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e
#[cfg(windows)]
pub const GUID_BALANCED: GUID = GUID {
    data1: 0x381b4222,
    data2: 0xf694,
    data3: 0x41f0,
    data4: [0x96, 0x85, 0xff, 0x5b, 0xb2, 0x60, 0xdf, 0x2e],
};

/// Processor Settings Subgroup GUID: 54533251-82be-4824-96c1-47b60b740d00
#[cfg(windows)]
pub const GUID_PROCESSOR_SETTINGS_SUBGROUP: GUID = GUID {
    data1: 0x54533251,
    data2: 0x82be,
    data3: 0x4824,
    data4: [0x96, 0xc1, 0x47, 0xb6, 0x0b, 0x74, 0x0d, 0x00],
};

/// Minimum Processor Cores (DC) GUID: 0cc5b647-c1df-4637-891a-dec35c318583
#[cfg(windows)]
pub const GUID_CPMINCORES: GUID = GUID {
    data1: 0x0cc5b647,
    data2: 0xc1df,
    data3: 0x4637,
    data4: [0x89, 0x1a, 0xde, 0xc3, 0x5c, 0x31, 0x85, 0x83],
};

/// Maximum Processor State GUID: bc5038f7-23e0-4960-96da-33abaf5935ec
#[cfg(windows)]
pub const GUID_PROCTHROTTLEMAX: GUID = GUID {
    data1: 0xbc5038f7,
    data2: 0x23e0,
    data3: 0x4960,
    data4: [0x96, 0xda, 0x33, 0xab, 0xaf, 0x59, 0x35, 0xec],
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryTelemetry {
    pub has_battery: bool,
    pub is_charging: bool,
    pub ac_connected: bool,
    pub battery_percent: u8,
    pub remaining_time_minutes: Option<u32>,
    pub eco_mode_active: bool,
    pub auto_switch_enabled: bool,
    pub refresh_rate_saver_enabled: bool,
    pub current_refresh_rate: u32,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryConfig {
    #[serde(default = "default_true")]
    pub auto_switch: bool,
    #[serde(default = "default_true")]
    pub refresh_rate_saver: bool,
    #[serde(default = "default_true")]
    pub dc_core_parking_enabled: bool,
    #[serde(default = "default_true")]
    pub turbo_limit_enabled: bool,
    #[serde(default)]
    pub saved_ac_refresh_rate: Option<u32>,
    #[serde(default = "default_turbo_max")]
    pub throttle_max_cpu_percent: u32,
    #[serde(default = "default_dc_parking")]
    pub dc_core_parking_percent: u32,
}

fn default_turbo_max() -> u32 {
    99
}

fn default_dc_parking() -> u32 {
    10
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            auto_switch: true,
            refresh_rate_saver: true,
            dc_core_parking_enabled: true,
            turbo_limit_enabled: true,
            saved_ac_refresh_rate: None,
            throttle_max_cpu_percent: 99,
            dc_core_parking_percent: 10,
        }
    }
}

pub type SharedBatteryConfig = Arc<RwLock<BatteryConfig>>;

fn battery_config_path() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("FerrumPulse").join("battery_config.json")
}

pub fn load_battery_config_from(path: &PathBuf) -> BatteryConfig {
    if !path.exists() {
        return BatteryConfig::default();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_battery_config_to(path: &PathBuf, cfg: &BatteryConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("Failed to serialize battery config: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("Failed to write battery config: {}", e))
}

pub fn load_battery_config() -> BatteryConfig {
    load_battery_config_from(&battery_config_path())
}

pub fn save_battery_config(cfg: &BatteryConfig) -> Result<(), String> {
    save_battery_config_to(&battery_config_path(), cfg)
}

/// Queries live battery status using GetSystemPowerStatus with persisted config values.
pub fn get_battery_telemetry_with_config(cfg: &BatteryConfig) -> BatteryTelemetry {
    #[cfg(windows)]
    {
        let mut sps = SYSTEM_POWER_STATUS::default();
        // SAFETY: GetSystemPowerStatus safely writes AC/battery state into a pre-initialized stack structure.
        let ok = unsafe { GetSystemPowerStatus(&mut sps) };

        if ok.is_err() {
            return BatteryTelemetry {
                has_battery: false,
                is_charging: false,
                ac_connected: true,
                battery_percent: 100,
                remaining_time_minutes: None,
                eco_mode_active: false,
                auto_switch_enabled: cfg.auto_switch,
                refresh_rate_saver_enabled: cfg.refresh_rate_saver,
                current_refresh_rate: get_current_display_refresh_rate().unwrap_or(60),
            };
        }

        let ac_connected = sps.ACLineStatus == 1;
        let is_charging = (sps.BatteryFlag & 8) != 0;
        let has_battery = sps.BatteryFlag != 128 && sps.BatteryFlag != 255;
        let battery_percent = if sps.BatteryLifePercent <= 100 {
            sps.BatteryLifePercent
        } else {
            100
        };

        let remaining_time_minutes = if sps.BatteryLifeTime != u32::MAX && sps.BatteryLifeTime > 0 {
            Some(sps.BatteryLifeTime / 60)
        } else {
            None
        };

        BatteryTelemetry {
            has_battery,
            is_charging,
            ac_connected,
            battery_percent,
            remaining_time_minutes,
            eco_mode_active: !ac_connected,
            auto_switch_enabled: cfg.auto_switch,
            refresh_rate_saver_enabled: cfg.refresh_rate_saver,
            current_refresh_rate: get_current_display_refresh_rate().unwrap_or(60),
        }
    }

    #[cfg(not(windows))]
    BatteryTelemetry {
        has_battery: true,
        is_charging: false,
        ac_connected: false,
        battery_percent: 85,
        remaining_time_minutes: Some(210),
        eco_mode_active: true,
        auto_switch_enabled: cfg.auto_switch,
        refresh_rate_saver_enabled: cfg.refresh_rate_saver,
        current_refresh_rate: 60,
    }
}

/// Queries live battery status using GetSystemPowerStatus
pub fn get_battery_telemetry() -> BatteryTelemetry {
    get_battery_telemetry_with_config(&load_battery_config())
}

/// Applies the Comprehensive Battery Eco Profile respecting persisted user policy toggles.
pub fn apply_battery_eco_profile_with_config(enable: bool, cfg: &BatteryConfig) -> Result<(), String> {
    #[cfg(windows)]
    {
        let target_scheme = if enable {
            GUID_POWER_SAVER
        } else {
            GUID_BALANCED
        };

        // 1. Activate Power Scheme
        // SAFETY: Switching active power plan using standard Windows Power Scheme GUIDs.
        unsafe {
            let res = PowerSetActiveScheme(None, Some(&target_scheme));
            if res != windows::Win32::Foundation::WIN32_ERROR(0) {
                let _ = PowerSetActiveScheme(None, Some(&GUID_BALANCED));
            }
        }
        // Align S0ix Power Mode Overlay as well on Modern Standby laptops
        let _ = crate::power::set_power_scheme(if enable { "balanced" } else { "balanced" });

        // 2. DC Min Cores Index (honors dc_core_parking_enabled)
        if cfg.dc_core_parking_enabled || !enable {
            let min_cores: u32 = if enable { cfg.dc_core_parking_percent } else { 50 };
            unsafe {
                let _ = PowerWriteDCValueIndex(
                    None,
                    &target_scheme,
                    Some(&GUID_PROCESSOR_SETTINGS_SUBGROUP),
                    Some(&GUID_CPMINCORES),
                    min_cores,
                );
            }
        }

        // 3. Max Processor State (honors turbo_limit_enabled)
        if cfg.turbo_limit_enabled || !enable {
            let max_cpu: u32 = if enable { cfg.throttle_max_cpu_percent } else { 100 };
            unsafe {
                let _ = PowerWriteDCValueIndex(
                    None,
                    &target_scheme,
                    Some(&GUID_PROCESSOR_SETTINGS_SUBGROUP),
                    Some(&GUID_PROCTHROTTLEMAX),
                    max_cpu,
                );
            }
        }

        // 4. Display refresh rate stepdown / restore (honors refresh_rate_saver)
        if cfg.refresh_rate_saver {
            if enable {
                let _ = set_display_refresh_rate(60);
            } else if let Some(saved_hz) = cfg.saved_ac_refresh_rate {
                if saved_hz > 60 {
                    let _ = set_display_refresh_rate(saved_hz);
                }
            }
        }

        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = (enable, cfg);
        Ok(())
    }
}

/// Applies the Comprehensive Battery Eco Profile (Power Saver, 10% DC Core Parking, 99% Max CPU)
pub fn apply_battery_eco_profile(enable: bool) -> Result<(), String> {
    let cfg = load_battery_config();
    apply_battery_eco_profile_with_config(enable, &cfg)
}

/// Deduplicates, filters (>= 30Hz), and sorts display refresh rates descending.
pub fn normalize_supported_refresh_rates(rates: Vec<u32>) -> Vec<u32> {
    let mut unique: Vec<u32> = rates.into_iter().filter(|&hz| hz >= 30).collect();
    unique.sort_unstable_by(|a, b| b.cmp(a));
    unique.dedup();
    unique
}

/// Enumerates all supported display refresh rates at the primary monitor's active resolution.
pub fn get_supported_refresh_rates() -> Vec<u32> {
    #[cfg(windows)]
    {
        let mut current = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        let cur_ok = unsafe { EnumDisplaySettingsW(None, ENUM_CURRENT_SETTINGS, &mut current) };
        if !cur_ok.as_bool() {
            return vec![60];
        }

        let target_w = current.dmPelsWidth;
        let target_h = current.dmPelsHeight;
        let target_bpp = current.dmBitsPerPel;

        let mut rates = Vec::new();
        let mut mode_idx = 0u32;
        loop {
            let mut mode = DEVMODEW {
                dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                ..Default::default()
            };
            let ok = unsafe {
                EnumDisplaySettingsW(
                    None,
                    windows::Win32::Graphics::Gdi::ENUM_DISPLAY_SETTINGS_MODE(mode_idx),
                    &mut mode,
                )
            };
            if !ok.as_bool() {
                break;
            }
            if mode.dmPelsWidth == target_w
                && mode.dmPelsHeight == target_h
                && (target_bpp == 0 || mode.dmBitsPerPel == target_bpp)
            {
                rates.push(mode.dmDisplayFrequency);
            }
            mode_idx += 1;
        }

        let normalized = normalize_supported_refresh_rates(rates);
        if normalized.is_empty() {
            vec![current.dmDisplayFrequency.max(60)]
        } else {
            normalized
        }
    }
    #[cfg(not(windows))]
    vec![240, 144, 120, 60]
}

/// Gets the current primary display refresh rate
pub fn get_current_display_refresh_rate() -> Option<u32> {
    #[cfg(windows)]
    {
        let mut devmode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };

        // SAFETY: EnumDisplaySettingsW queries display frequency into a DEVMODEW structure with pre-sized dmSize.
        let ok = unsafe {
            EnumDisplaySettingsW(None, ENUM_CURRENT_SETTINGS, &mut devmode)
        };

        if ok.as_bool() && devmode.dmDisplayFrequency > 0 {
            Some(devmode.dmDisplayFrequency)
        } else {
            None
        }
    }
    #[cfg(not(windows))]
    Some(60)
}

/// Changes the primary display refresh rate (e.g. 60Hz on battery)
pub fn set_display_refresh_rate(target_hz: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        let mut devmode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };

        // SAFETY: Querying current display configuration into sized DEVMODEW.
        let ok = unsafe {
            EnumDisplaySettingsW(None, ENUM_CURRENT_SETTINGS, &mut devmode)
        };

        if !ok.as_bool() {
            return Err("Failed to query display settings".into());
        }

        if devmode.dmDisplayFrequency == target_hz {
            return Ok(()); // Already at target rate
        }

        devmode.dmDisplayFrequency = target_hz;
        devmode.dmFields = windows::Win32::Graphics::Gdi::DEVMODE_FIELD_FLAGS(0x00400000); // DM_DISPLAYFREQUENCY

        // SAFETY: Updating primary display frequency with CDS_UPDATEREGISTRY flag.
        let res = unsafe {
            ChangeDisplaySettingsExW(None, Some(&devmode), None, CDS_UPDATEREGISTRY, None)
        };

        if res == DISP_CHANGE_SUCCESSFUL {
            Ok(())
        } else {
            Err(format!("ChangeDisplaySettingsExW failed with code {:?}", res))
        }
    }
    #[cfg(not(windows))]
    {
        let _ = target_hz;
        Ok(())
    }
}

/// Enforces failsafe controls when transitioning to battery power or during Modern Standby / Sleep:
/// 1. Reverts CPU Core Parking from unparked (100%) back to stock (50%).
/// 2. Switches active Windows power scheme back to Balanced.
/// 3. Releases high-precision multimedia timer lock (allowing deep processor C-states like C6/C7/C10/DRIPS).
pub fn enforce_battery_and_standby_failsafe() {
    #[cfg(windows)]
    {
        // 1. Relinquish multimedia timer precision request to allow deep CPU C-states (preventing closed-bag overheating)
        crate::process::unlock_high_resolution_timer();

        // 2. Revert Core Parking from 100% (unparked) back to stock 50%
        let _ = crate::power::set_core_unparking(false);

        // 3. Revert power scheme to Balanced
        let _ = crate::power::set_power_scheme("balanced");
    }
}
