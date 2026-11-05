use serde::{Deserialize, Serialize};

#[cfg(windows)]
use windows::core::{w, PCWSTR};
#[cfg(windows)]
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, REG_DWORD, REG_SZ, REG_OPTION_NON_VOLATILE,
};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETMOUSE,
};
#[cfg(windows)]
use windows::Win32::System::Services::{
    ChangeServiceConfigW, CloseServiceHandle, ControlService, OpenSCManagerW, OpenServiceW,
    QueryServiceConfigW, ENUM_SERVICE_TYPE, QUERY_SERVICE_CONFIGW, SC_MANAGER_ALL_ACCESS,
    SC_MANAGER_CONNECT,
    SERVICE_ALL_ACCESS, SERVICE_CONTROL_STOP, SERVICE_DEMAND_START, SERVICE_DISABLED,
    SERVICE_ERROR, SERVICE_NO_CHANGE, SERVICE_QUERY_CONFIG, SERVICE_QUERY_STATUS,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweaksStatus {
    pub diagtrack_disabled: bool,
    pub gamedvr_disabled: bool,
    pub network_pacing_optimized: bool,
    pub global_timer_enabled: bool,
    pub raw_mouse_enabled: bool,
    pub autostart_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakActionResult {
    pub success: bool,
    pub message: String,
}

/// Queries current status of all safe optimization presets.
pub fn get_tweaks_status() -> TweaksStatus {
    TweaksStatus {
        diagtrack_disabled: is_diagtrack_disabled(),
        gamedvr_disabled: is_gamedvr_disabled(),
        network_pacing_optimized: is_network_pacing_optimized(),
        global_timer_enabled: is_global_timer_resolution_enabled(),
        raw_mouse_enabled: is_raw_mouse_input_enabled(),
        autostart_enabled: is_autostart_enabled(),
    }
}

// ---------------------------------------------------------------------------
// 1. DiagTrack (Connected User Experiences and Telemetry) Service
// ---------------------------------------------------------------------------

pub fn is_diagtrack_disabled() -> bool {
    #[cfg(windows)]
    {
        // SAFETY: Opening SCM with read/connect permissions (works without admin elevation)
        let scm = unsafe { OpenSCManagerW(None, None, SC_MANAGER_CONNECT) };
        if scm.is_err() {
            return false;
        }
        let scm_handle = scm.unwrap();

        let svc = unsafe {
            OpenServiceW(
                scm_handle,
                w!("DiagTrack"),
                SERVICE_QUERY_CONFIG | SERVICE_QUERY_STATUS,
            )
        };

        if svc.is_err() {
            unsafe {
                let _ = CloseServiceHandle(scm_handle);
            };
            return false;
        }
        let svc_handle = svc.unwrap();

        let mut bytes_needed: u32 = 0;
        // First call to determine buffer size
        let _ = unsafe {
            QueryServiceConfigW(svc_handle, None, 0, &mut bytes_needed)
        };

        let mut is_disabled = false;
        if bytes_needed > 0 {
            let mut buf: Vec<u8> = vec![0; bytes_needed as usize];
            let config_ptr = buf.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW;
            let query_res = unsafe {
                QueryServiceConfigW(svc_handle, Some(config_ptr), bytes_needed, &mut bytes_needed)
            };

            if query_res.is_ok() {
                let config = unsafe { &*config_ptr };
                is_disabled = config.dwStartType == SERVICE_DISABLED;
            }
        }

        unsafe {
            let _ = CloseServiceHandle(svc_handle);
            let _ = CloseServiceHandle(scm_handle);
        };

        is_disabled
    }

    #[cfg(not(windows))]
    {
        false
    }
}

pub fn toggle_diagtrack(disable: bool) -> TweakActionResult {
    #[cfg(windows)]
    {
        // SAFETY: Opening SCM with admin permissions
        let scm = unsafe { OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS) };
        if scm.is_err() {
            return TweakActionResult {
                success: false,
                message: "Access Denied: Run FerrumPulse as Administrator.".into(),
            };
        }
        let scm_handle = scm.unwrap();

        let svc = unsafe {
            OpenServiceW(
                scm_handle,
                w!("DiagTrack"),
                SERVICE_ALL_ACCESS,
            )
        };

        if svc.is_err() {
            unsafe {
                let _ = CloseServiceHandle(scm_handle);
            };
            return TweakActionResult {
                success: false,
                message: "DiagTrack service not found or access restricted.".into(),
            };
        }
        let svc_handle = svc.unwrap();

        let target_start_type = if disable {
            SERVICE_DISABLED
        } else {
            SERVICE_DEMAND_START // Safe default for Windows telemetry revert
        };

        // SAFETY: Updating service startup type with typed ENUM_SERVICE_TYPE & SERVICE_ERROR
        let cfg_res = unsafe {
            ChangeServiceConfigW(
                svc_handle,
                ENUM_SERVICE_TYPE(SERVICE_NO_CHANGE),
                target_start_type,
                SERVICE_ERROR(SERVICE_NO_CHANGE),
                PCWSTR::null(),
                PCWSTR::null(),
                None,
                PCWSTR::null(),
                PCWSTR::null(),
                PCWSTR::null(),
                PCWSTR::null(),
            )
        };

        if disable {
            let mut status = windows::Win32::System::Services::SERVICE_STATUS::default();
            // SAFETY: Stopping the running service immediately
            let _ = unsafe { ControlService(svc_handle, SERVICE_CONTROL_STOP, &mut status) };
        }

        unsafe {
            let _ = CloseServiceHandle(svc_handle);
            let _ = CloseServiceHandle(scm_handle);
        };

        if cfg_res.is_ok() {
            TweakActionResult {
                success: true,
                message: if disable {
                    "DiagTrack telemetry service stopped & disabled.".into()
                } else {
                    "DiagTrack telemetry restored to default (Manual start).".into()
                },
            }
        } else {
            TweakActionResult {
                success: false,
                message: format!("Failed to update DiagTrack service: {:?}", cfg_res),
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = disable;
        TweakActionResult {
            success: true,
            message: "Mock DiagTrack toggle successful.".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Windows Game Bar DVR (AppCaptureEnabled & GameDVR_Enabled)
// ---------------------------------------------------------------------------

pub fn is_gamedvr_disabled() -> bool {
    #[cfg(windows)]
    {
        let app_capture = read_registry_dword(
            HKEY_CURRENT_USER,
            w!("System\\GameConfigStore"),
            w!("AppCaptureEnabled"),
        )
        .unwrap_or(1);

        let gamedvr = read_registry_dword(
            HKEY_CURRENT_USER,
            w!("System\\GameConfigStore"),
            w!("GameDVR_Enabled"),
        )
        .unwrap_or(1);

        app_capture == 0 && gamedvr == 0
    }

    #[cfg(not(windows))]
    {
        false
    }
}

pub fn toggle_gamedvr(disable: bool) -> TweakActionResult {
    #[cfg(windows)]
    {
        let val: u32 = if disable { 0 } else { 1 };

        let res1 = write_registry_dword(
            HKEY_CURRENT_USER,
            w!("System\\GameConfigStore"),
            w!("AppCaptureEnabled"),
            val,
        );

        let res2 = write_registry_dword(
            HKEY_CURRENT_USER,
            w!("System\\GameConfigStore"),
            w!("GameDVR_Enabled"),
            val,
        );

        let res3 = write_registry_dword(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\GameDVR"),
            w!("AppCaptureEnabled"),
            val,
        );

        if res1.is_ok() && res2.is_ok() && res3.is_ok() {
            TweakActionResult {
                success: true,
                message: if disable {
                    "Game Bar DVR & background recording disabled.".into()
                } else {
                    "Game Bar DVR restored to Windows default.".into()
                },
            }
        } else {
            TweakActionResult {
                success: false,
                message: "Failed to modify GameDVR registry entries.".into(),
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = disable;
        TweakActionResult {
            success: true,
            message: "Mock GameDVR toggle successful.".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Network Gaming Pacing (NetworkThrottlingIndex & SystemResponsiveness)
// ---------------------------------------------------------------------------

pub fn is_network_pacing_optimized() -> bool {
    #[cfg(windows)]
    {
        let throttling_index = read_registry_dword(
            HKEY_LOCAL_MACHINE,
            w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile"),
            w!("NetworkThrottlingIndex"),
        )
        .unwrap_or(10);

        let responsiveness = read_registry_dword(
            HKEY_LOCAL_MACHINE,
            w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile"),
            w!("SystemResponsiveness"),
        )
        .unwrap_or(20);

        throttling_index == 0xffffffff && responsiveness == 0
    }

    #[cfg(not(windows))]
    {
        false
    }
}

pub fn toggle_network_pacing(optimize: bool) -> TweakActionResult {
    #[cfg(windows)]
    {
        let (throttling_val, resp_val) = if optimize {
            (0xffffffffu32, 0u32) // Unthrottled network, 100% responsiveness to gaming
        } else {
            (10u32, 20u32) // Default Windows stock settings
        };

        let res1 = write_registry_dword(
            HKEY_LOCAL_MACHINE,
            w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile"),
            w!("NetworkThrottlingIndex"),
            throttling_val,
        );

        let res2 = write_registry_dword(
            HKEY_LOCAL_MACHINE,
            w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Multimedia\\SystemProfile"),
            w!("SystemResponsiveness"),
            resp_val,
        );

        if res1.is_ok() && res2.is_ok() {
            TweakActionResult {
                success: true,
                message: if optimize {
                    "Network Throttling removed & Multimedia Pacing set to 100% responsiveness.".into()
                } else {
                    "Network Throttling & Multimedia Pacing restored to stock defaults.".into()
                },
            }
        } else {
            TweakActionResult {
                success: false,
                message: "Access Denied modifying Multimedia SystemProfile. Requires Administrator.".into(),
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = optimize;
        TweakActionResult {
            success: true,
            message: "Mock Network Pacing toggle successful.".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Win32 Registry Helpers (RAII-safe)
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn read_registry_dword(root: HKEY, subkey: PCWSTR, value_name: PCWSTR) -> Option<u32> {
    let mut hkey = HKEY::default();
    // SAFETY: RegOpenKeyExW safely opens registry key for read access
    let res = unsafe { RegOpenKeyExW(root, subkey, 0, KEY_READ, &mut hkey) };
    if res.is_err() {
        return None;
    }

    let mut data: u32 = 0;
    let mut data_size = std::mem::size_of::<u32>() as u32;
    let mut val_type = REG_DWORD;

    // SAFETY: Querying 32-bit DWORD into sized variable
    let query_res = unsafe {
        RegQueryValueExW(
            hkey,
            value_name,
            None,
            Some(&mut val_type),
            Some(&mut data as *mut u32 as _),
            Some(&mut data_size),
        )
    };

    unsafe {
        let _ = RegCloseKey(hkey);
    };

    if query_res.is_ok() {
        Some(data)
    } else {
        None
    }
}

#[cfg(windows)]
fn write_registry_dword(root: HKEY, subkey: PCWSTR, value_name: PCWSTR, value: u32) -> Result<(), String> {
    let mut hkey = HKEY::default();
    // SAFETY: RegCreateKeyExW opens or creates subkey with write permissions
    let res = unsafe {
        RegCreateKeyExW(
            root,
            subkey,
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut hkey,
            None,
        )
    };

    if res.is_err() {
        return Err(format!("RegCreateKeyExW failed: {:?}", res));
    }

    // SAFETY: Writing 4-byte DWORD value
    let set_res = unsafe {
        RegSetValueExW(
            hkey,
            value_name,
            0,
            REG_DWORD,
            Some(std::slice::from_raw_parts(
                &value as *const u32 as *const u8,
                std::mem::size_of::<u32>(),
            )),
        )
    };

    unsafe {
        let _ = RegCloseKey(hkey);
    };

    if set_res.is_ok() {
        Ok(())
    } else {
        Err(format!("RegSetValueExW failed: {:?}", set_res))
    }
}

#[cfg(windows)]
fn read_registry_sz(root: HKEY, subkey: PCWSTR, value_name: PCWSTR) -> Option<String> {
    let mut hkey = HKEY::default();
    // SAFETY: Opening registry subkey with read-only permissions
    let res = unsafe { RegOpenKeyExW(root, subkey, 0, KEY_READ, &mut hkey) };
    if res.is_err() {
        return None;
    }

    let mut buf = [0u16; 512];
    let mut data_size = (buf.len() * 2) as u32;
    let mut val_type = REG_SZ;

    // SAFETY: Reading UTF-16 registry string into stack-allocated fixed buffer
    let query_res = unsafe {
        RegQueryValueExW(
            hkey,
            value_name,
            None,
            Some(&mut val_type),
            Some(buf.as_mut_ptr() as _),
            Some(&mut data_size),
        )
    };

    // SAFETY: Closing open registry key handle
    unsafe {
        let _ = RegCloseKey(hkey);
    };

    if query_res.is_ok() {
        let len = (data_size / 2) as usize;
        let trimmed_len = if len > 0 && buf[len - 1] == 0 { len - 1 } else { len };
        Some(String::from_utf16_lossy(&buf[..trimmed_len]))
    } else {
        None
    }
}

#[cfg(windows)]
fn write_registry_sz(root: HKEY, subkey: PCWSTR, value_name: PCWSTR, value: &str) -> Result<(), String> {
    let mut hkey = HKEY::default();
    // SAFETY: Opening or creating registry subkey with set-value permissions
    let res = unsafe {
        RegCreateKeyExW(
            root,
            subkey,
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut hkey,
            None,
        )
    };
    if res.is_err() {
        return Err(format!("RegCreateKeyExW failed: {:?}", res));
    }

    let wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: Writing null-terminated UTF-16 byte slice to the open registry key
    let set_res = unsafe {
        RegSetValueExW(
            hkey,
            value_name,
            0,
            REG_SZ,
            Some(std::slice::from_raw_parts(
                wide.as_ptr() as *const u8,
                wide.len() * 2,
            )),
        )
    };

    // SAFETY: Closing open registry key handle
    unsafe {
        let _ = RegCloseKey(hkey);
    };

    if set_res.is_ok() {
        Ok(())
    } else {
        Err(format!("RegSetValueExW failed: {:?}", set_res))
    }
}

#[cfg(windows)]
fn delete_registry_value(root: HKEY, subkey: PCWSTR, value_name: PCWSTR) -> Result<(), String> {
    let mut hkey = HKEY::default();
    // SAFETY: Opening registry subkey with set-value permissions
    let res = unsafe { RegOpenKeyExW(root, subkey, 0, KEY_SET_VALUE, &mut hkey) };
    if res.is_err() {
        return Err(format!("RegOpenKeyExW failed: {:?}", res));
    }

    // SAFETY: Deleting named value under open key handle
    let del_res = unsafe { RegDeleteValueW(hkey, value_name) };
    // SAFETY: Closing open registry key handle
    unsafe {
        let _ = RegCloseKey(hkey);
    };

    if del_res.is_ok() {
        Ok(())
    } else {
        Err(format!("RegDeleteValueW failed: {:?}", del_res))
    }
}

// ---------------------------------------------------------------------------
// 4. Global Timer Resolution (Windows 11 Micro-Stutter Fix)
// ---------------------------------------------------------------------------

pub fn is_global_timer_resolution_enabled() -> bool {
    #[cfg(windows)]
    {
        read_registry_dword(
            HKEY_LOCAL_MACHINE,
            w!("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel"),
            w!("GlobalTimerResolutionRequests"),
        ) == Some(1)
    }
    #[cfg(not(windows))]
    false
}

pub fn toggle_global_timer_resolution(enable: bool) -> TweakActionResult {
    #[cfg(windows)]
    {
        let val: u32 = if enable { 1 } else { 0 };
        let res = write_registry_dword(
            HKEY_LOCAL_MACHINE,
            w!("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\kernel"),
            w!("GlobalTimerResolutionRequests"),
            val,
        );
        match res {
            Ok(_) => TweakActionResult {
                success: true,
                message: if enable {
                    "Global timer resolution enabled (Windows 11 1.0ms timer pacing active).".into()
                } else {
                    "Global timer resolution restored to default.".into()
                },
            },
            Err(_) => TweakActionResult {
                success: false,
                message: "Administrator privileges required to toggle GlobalTimerResolutionRequests.".into(),
            },
        }
    }
    #[cfg(not(windows))]
    {
        let _ = enable;
        TweakActionResult {
            success: true,
            message: "Mock global timer toggle successful.".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Raw Esports 1:1 Mouse Input (Curve Acceleration Neutralization)
// ---------------------------------------------------------------------------

pub fn is_raw_mouse_input_enabled() -> bool {
    #[cfg(windows)]
    {
        let speed = read_registry_sz(HKEY_CURRENT_USER, w!("Control Panel\\Mouse"), w!("MouseSpeed"));
        speed.as_deref() == Some("0")
    }
    #[cfg(not(windows))]
    false
}

pub fn toggle_raw_mouse_input(enable: bool) -> TweakActionResult {
    #[cfg(windows)]
    {
        let (speed, t1, t2, int_params) = if enable {
            ("0", "0", "0", [0i32, 0i32, 0i32])
        } else {
            ("1", "6", "10", [6i32, 10i32, 1i32])
        };

        let _ = write_registry_sz(HKEY_CURRENT_USER, w!("Control Panel\\Mouse"), w!("MouseSpeed"), speed);
        let _ = write_registry_sz(HKEY_CURRENT_USER, w!("Control Panel\\Mouse"), w!("MouseThreshold1"), t1);
        let _ = write_registry_sz(HKEY_CURRENT_USER, w!("Control Panel\\Mouse"), w!("MouseThreshold2"), t2);

        // Apply immediately to system live session
        // SAFETY: SystemParametersInfoW with SPI_SETMOUSE safely reads an array of 3 integers
        // from a valid stack allocation int_params ([Threshold1, Threshold2, MouseSpeed]).
        unsafe {
            let _ = SystemParametersInfoW(
                SPI_SETMOUSE,
                0,
                Some(int_params.as_ptr() as _),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            );
        }

        TweakActionResult {
            success: true,
            message: if enable {
                "Raw 1:1 mouse input enabled (cursor curve acceleration disabled).".into()
            } else {
                "Windows default mouse curve acceleration restored.".into()
            },
        }
    }
    #[cfg(not(windows))]
    {
        let _ = enable;
        TweakActionResult {
            success: true,
            message: "Mock raw mouse input toggle successful.".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Windows Startup / Autostart Daemon Toggle
// ---------------------------------------------------------------------------

pub fn is_autostart_enabled() -> bool {
    #[cfg(windows)]
    {
        read_registry_sz(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            w!("FerrumPulse"),
        )
        .is_some()
    }
    #[cfg(not(windows))]
    false
}

pub fn toggle_autostart(enable: bool) -> TweakActionResult {
    #[cfg(windows)]
    {
        if enable {
            if let Ok(exe_path) = std::env::current_exe() {
                let path_str = format!("\"{}\" --minimized", exe_path.to_string_lossy());
                let res = write_registry_sz(
                    HKEY_CURRENT_USER,
                    w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
                    w!("FerrumPulse"),
                    &path_str,
                );
                match res {
                    Ok(_) => TweakActionResult {
                        success: true,
                        message: "FerrumPulse configured to start automatically on Windows boot.".into(),
                    },
                    Err(e) => TweakActionResult {
                        success: false,
                        message: format!("Failed to register startup item: {}", e),
                    },
                }
            } else {
                TweakActionResult {
                    success: false,
                    message: "Unable to determine application executable path.".into(),
                }
            }
        } else {
            let _ = delete_registry_value(
                HKEY_CURRENT_USER,
                w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
                w!("FerrumPulse"),
            );
            TweakActionResult {
                success: true,
                message: "FerrumPulse removed from Windows startup.".into(),
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = enable;
        TweakActionResult {
            success: true,
            message: "Mock autostart toggle successful.".into(),
        }
    }
}
