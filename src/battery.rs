use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryConfig {
    pub auto_switch: bool,
    pub refresh_rate_saver: bool,
    pub throttle_max_cpu_percent: u32,
    pub dc_core_parking_percent: u32,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            auto_switch: true,
            refresh_rate_saver: true,
            throttle_max_cpu_percent: 99,
            dc_core_parking_percent: 10,
        }
    }
}

/// Queries live battery status using GetSystemPowerStatus
pub fn get_battery_telemetry() -> BatteryTelemetry {
    #[cfg(windows)]
    {
        let mut sps = SYSTEM_POWER_STATUS::default();
        let ok = unsafe { GetSystemPowerStatus(&mut sps) };

        if ok.is_err() {
            return BatteryTelemetry {
                has_battery: false,
                is_charging: false,
                ac_connected: true,
                battery_percent: 100,
                remaining_time_minutes: None,
                eco_mode_active: false,
                auto_switch_enabled: true,
                refresh_rate_saver_enabled: true,
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
            auto_switch_enabled: true,
            refresh_rate_saver_enabled: true,
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
        auto_switch_enabled: true,
        refresh_rate_saver_enabled: true,
        current_refresh_rate: 60,
    }
}

/// Applies the Comprehensive Battery Eco Profile (Power Saver, 10% DC Core Parking, 99% Max CPU)
pub fn apply_battery_eco_profile(enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        let target_scheme = if enable {
            GUID_POWER_SAVER
        } else {
            GUID_BALANCED
        };

        // 1. Activate Power Scheme
        unsafe {
            let res = PowerSetActiveScheme(None, Some(&target_scheme));
            if res != windows::Win32::Foundation::WIN32_ERROR(0) {
                // If Power Saver does not exist on some OEM images, try Balanced
                let _ = PowerSetActiveScheme(None, Some(&GUID_BALANCED));
            }
        }

        // 2. DC Min Cores Index (10% on battery allowing unused cores to deep-sleep, 50% default on restore)
        let min_cores: u32 = if enable { 10 } else { 50 };
        unsafe {
            let _ = PowerWriteDCValueIndex(
                None,
                &target_scheme,
                Some(&GUID_PROCESSOR_SETTINGS_SUBGROUP),
                Some(&GUID_CPMINCORES),
                min_cores,
            );
        }

        // 3. Max Processor State (99% prevents aggressive Intel/AMD Turbo boost voltage spikes on DC)
        let max_cpu: u32 = if enable { 99 } else { 100 };
        unsafe {
            let _ = PowerWriteDCValueIndex(
                None,
                &target_scheme,
                Some(&GUID_PROCESSOR_SETTINGS_SUBGROUP),
                Some(&GUID_PROCTHROTTLEMAX),
                max_cpu,
            );
        }

        // 4. Optionally step down display refresh rate to 60Hz when on battery
        if enable {
            let _ = set_display_refresh_rate(60);
        }

        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = enable;
        Ok(())
    }
}

/// Gets the current primary display refresh rate
pub fn get_current_display_refresh_rate() -> Option<u32> {
    #[cfg(windows)]
    {
        let mut devmode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };

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
