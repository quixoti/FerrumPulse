use serde::{Deserialize, Serialize};

#[cfg(windows)]
use windows::core::GUID;
#[cfg(windows)]
use windows::Win32::Foundation::LocalFree;
#[cfg(windows)]
use windows::Win32::System::Power::{
    PowerGetActiveScheme, PowerReadACValueIndex, PowerSetActiveScheme, PowerWriteACValueIndex,
};

/// Processor Settings Subgroup GUID: 54533251-82be-4824-96c1-47b60b740d00
#[cfg(windows)]
pub const GUID_PROCESSOR_SETTINGS_SUBGROUP: GUID = GUID {
    data1: 0x54533251,
    data2: 0x82be,
    data3: 0x4824,
    data4: [0x96, 0xc1, 0x47, 0xb6, 0x0b, 0x74, 0x0d, 0x00],
};

/// Core Parking Minimum Cores (AC) GUID: 0cc5b647-c1df-4637-891a-dec35c318583
#[cfg(windows)]
pub const GUID_CPMINCORES: GUID = GUID {
    data1: 0x0cc5b647,
    data2: 0xc1df,
    data3: 0x4637,
    data4: [0x89, 0x1a, 0xde, 0xc3, 0x5c, 0x31, 0x85, 0x83],
};

/// High Performance Power Scheme GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c
#[cfg(windows)]
pub const GUID_HIGH_PERFORMANCE: GUID = GUID {
    data1: 0x8c5e7fda,
    data2: 0xe8bf,
    data3: 0x4a96,
    data4: [0x9a, 0x85, 0xa6, 0xe2, 0x3a, 0x8c, 0x63, 0x5c],
};

/// Balanced Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e
#[cfg(windows)]
pub const GUID_BALANCED: GUID = GUID {
    data1: 0x381b4222,
    data2: 0xf694,
    data3: 0x41f0,
    data4: [0x96, 0x85, 0xff, 0x5b, 0xb2, 0x60, 0xdf, 0x2e],
};

/// Ultimate Performance Power Scheme GUID: e9a42b02-d5df-448d-aa00-03f14749eb61
#[cfg(windows)]
pub const GUID_ULTIMATE_PERFORMANCE: GUID = GUID {
    data1: 0xe9a42b02,
    data2: 0xd5df,
    data3: 0x448d,
    data4: [0xaa, 0x00, 0x03, 0xf1, 0x47, 0x49, 0xeb, 0x61],
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerStatus {
    pub active_scheme_name: String,
    pub active_scheme_guid: String,
    pub min_cores_ac_percent: u32,
    pub is_unparked: bool,
}

/// Retrieves the active Windows power scheme and current CPU core parking percentage.
pub fn get_power_status() -> Result<PowerStatus, String> {
    #[cfg(windows)]
    {
        let mut active_guid_ptr: *mut GUID = std::ptr::null_mut();

        // SAFETY: PowerGetActiveScheme allocates a GUID block for active power scheme
        // which we read and then properly free using LocalFree to avoid leaks.
        let status = unsafe { PowerGetActiveScheme(None, &mut active_guid_ptr) };
        if status.is_err() || active_guid_ptr.is_null() {
            return Err("Failed to query active Windows power scheme.".into());
        }

        let active_guid = unsafe { *active_guid_ptr };
        unsafe {
            let _ = LocalFree(windows::Win32::Foundation::HLOCAL(active_guid_ptr as _));
        };

        let mut min_cores_percent: u32 = 0;
        // SAFETY: Reading AC power value index for CPU min cores parking
        let read_status = unsafe {
            PowerReadACValueIndex(
                None,
                Some(&active_guid),
                Some(&GUID_PROCESSOR_SETTINGS_SUBGROUP),
                Some(&GUID_CPMINCORES),
                &mut min_cores_percent,
            )
        };

        if read_status.is_err() {
            // Default fallback if key not readable
            min_cores_percent = 50;
        }

        let guid_str = format!(
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            active_guid.data1,
            active_guid.data2,
            active_guid.data3,
            active_guid.data4[0],
            active_guid.data4[1],
            active_guid.data4[2],
            active_guid.data4[3],
            active_guid.data4[4],
            active_guid.data4[5],
            active_guid.data4[6],
            active_guid.data4[7],
        );

        let scheme_name = match active_guid {
            g if g == GUID_HIGH_PERFORMANCE => "High Performance".to_string(),
            g if g == GUID_BALANCED => "Balanced".to_string(),
            g if g == GUID_ULTIMATE_PERFORMANCE => "Ultimate Performance".to_string(),
            _ => "Custom Power Plan".to_string(),
        };

        Ok(PowerStatus {
            active_scheme_name: scheme_name,
            active_scheme_guid: guid_str,
            min_cores_ac_percent: min_cores_percent,
            is_unparked: min_cores_percent >= 100,
        })
    }

    #[cfg(not(windows))]
    {
        Ok(PowerStatus {
            active_scheme_name: "Mock Balanced".into(),
            active_scheme_guid: "381b4222-f694-41f0-9685-ff5bb260df2e".into(),
            min_cores_ac_percent: 100,
            is_unparked: true,
        })
    }
}

/// Sets the Core Parking Min Cores (AC) value index (100% = unparked, 50% = default).
pub fn set_core_unparking(unpark: bool) -> Result<u32, String> {
    let target_percent: u32 = if unpark { 100 } else { 50 };

    #[cfg(windows)]
    {
        let mut active_guid_ptr: *mut GUID = std::ptr::null_mut();

        // SAFETY: Querying current active scheme to update parking index
        let status = unsafe { PowerGetActiveScheme(None, &mut active_guid_ptr) };
        if status.is_err() || active_guid_ptr.is_null() {
            return Err("Unable to retrieve active power scheme.".into());
        }

        let active_guid = unsafe { *active_guid_ptr };
        unsafe {
            let _ = LocalFree(windows::Win32::Foundation::HLOCAL(active_guid_ptr as _));
        };

        // SAFETY: Writing CPU core parking AC index directly to active scheme
        let write_status = unsafe {
            PowerWriteACValueIndex(
                None,
                &active_guid,
                Some(&GUID_PROCESSOR_SETTINGS_SUBGROUP),
                Some(&GUID_CPMINCORES),
                target_percent,
            )
        };

        if write_status.is_err() {
            return Err(format!(
                "Failed to write Core Parking index (error: {:?})",
                write_status
            ));
        }

        // SAFETY: Applying active scheme changes immediately so governor updates
        let apply_status = unsafe { PowerSetActiveScheme(None, Some(&active_guid)) };
        if apply_status.is_err() {
            return Err("Failed to apply updated power scheme.".into());
        }

        Ok(target_percent)
    }

    #[cfg(not(windows))]
    {
        Ok(target_percent)
    }
}

/// Switches active Windows Power Scheme (e.g. High Performance or Balanced).
pub fn set_power_scheme(scheme_type: &str) -> Result<String, String> {
    #[cfg(windows)]
    {
        let target_guid = match scheme_type.to_lowercase().as_str() {
            "high" | "high_performance" => GUID_HIGH_PERFORMANCE,
            "balanced" => GUID_BALANCED,
            "ultimate" => GUID_ULTIMATE_PERFORMANCE,
            _ => return Err("Invalid power scheme requested.".into()),
        };

        // SAFETY: Setting active scheme by known Windows GUID
        let apply_status = unsafe { PowerSetActiveScheme(None, Some(&target_guid)) };
        if apply_status.is_err() {
            return Err(format!(
                "Could not activate requested power plan: {:?}",
                apply_status
            ));
        }

        Ok(format!("Successfully activated {} scheme.", scheme_type))
    }

    #[cfg(not(windows))]
    {
        Ok(format!("Mock activated {} scheme.", scheme_type))
    }
}

