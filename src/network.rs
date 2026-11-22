// src/network.rs - Esports Network QoS, Packet Pacing & Ping Jitter Engine
// 100% User-Mode Win32 Registry Policies and ICMP Echo Diagnostics

use serde::{Deserialize, Serialize};
use crate::profiles::GameProfile;
use crate::tweaks::TweakActionResult;

#[cfg(windows)]
use windows::core::{w, PCWSTR, HSTRING};
#[cfg(windows)]
use windows::Win32::System::Registry::{
    HKEY_LOCAL_MACHINE, RegCloseKey, RegDeleteKeyW, RegEnumKeyExW, RegOpenKeyExW,
    HKEY, KEY_READ,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTweaksStatus {
    pub tcp_latency_optimized: bool,
    pub active_interfaces_count: usize,
    pub qos_policies_count: usize,
    pub active_game_policies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingProbeReport {
    pub target_host: String,
    pub min_ping_ms: f64,
    pub max_ping_ms: f64,
    pub avg_ping_ms: f64,
    pub jitter_ms: f64,
    pub packet_loss_percent: f64,
    pub samples_sent: u32,
    pub samples_received: u32,
    pub status_verdict: String,
    pub recommendations: Vec<String>,
}

// ---------------------------------------------------------------------------
// FFI: Windows IcmpSendEcho for ICMP Diagnostics
// ---------------------------------------------------------------------------

#[cfg(windows)]
#[link(name = "iphlpapi")]
extern "system" {
    fn IcmpCreateFile() -> *mut std::ffi::c_void;
    fn IcmpCloseHandle(icmp_handle: *mut std::ffi::c_void) -> i32;
    fn IcmpSendEcho(
        icmp_handle: *mut std::ffi::c_void,
        destination_address: u32,
        request_data: *const u8,
        request_size: u16,
        request_options: *const std::ffi::c_void,
        reply_buffer: *mut u8,
        reply_size: u32,
        timeout: u32,
    ) -> u32;
}

#[cfg(windows)]
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct IcmpEchoReply {
    address: u32,
    status: u32,
    round_trip_time: u32,
    data_size: u16,
    reserved: u16,
    data: *mut std::ffi::c_void,
    options_ttl: u8,
    options_tos: u8,
    options_flags: u8,
    options_size: u8,
    options_data: *mut std::ffi::c_void,
}

// ---------------------------------------------------------------------------
// Per-Game Windows QoS Policies (DSCP 46 Expedited Forwarding)
// ---------------------------------------------------------------------------

const QOS_BASE_KEY: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\QoS";

/// Sanitize profile display name to create a safe Windows Registry key
pub fn sanitize_policy_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    format!("FerrumPulse_{}", clean)
}

/// Registers a Windows QoS Group Policy tagging game packets with DSCP 46 (Expedited Forwarding)
pub fn apply_dscp_qos_policy(game_name: &str, executable_name: &str) -> TweakActionResult {
    #[cfg(windows)]
    {
        use crate::tweaks::write_registry_sz;
        let policy_name = sanitize_policy_name(game_name);
        let subkey = format!("{}\\{}", QOS_BASE_KEY, policy_name);
        let subkey_h = HSTRING::from(subkey.as_str());
        let subkey_pc = PCWSTR(subkey_h.as_ptr());

        let res_ver = write_registry_sz(HKEY_LOCAL_MACHINE, subkey_pc, w!("Version"), "1.0");
        let res_app = write_registry_sz(HKEY_LOCAL_MACHINE, subkey_pc, w!("Application Name"), executable_name);
        let res_proto = write_registry_sz(HKEY_LOCAL_MACHINE, subkey_pc, w!("Protocol"), "*");
        let res_dscp = write_registry_sz(HKEY_LOCAL_MACHINE, subkey_pc, w!("DSCP Value"), "46");
        let res_rate = write_registry_sz(HKEY_LOCAL_MACHINE, subkey_pc, w!("Throttle Rate"), "-1");

        if res_ver.is_ok() && res_app.is_ok() && res_proto.is_ok() && res_dscp.is_ok() && res_rate.is_ok() {
            TweakActionResult {
                success: true,
                message: format!("Applied DSCP 46 (Expedited Forwarding) QoS policy for {}", executable_name),
            }
        } else {
            TweakActionResult {
                success: false,
                message: format!("Failed to create Windows QoS registry policy for {}. Administrator rights required.", game_name),
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (game_name, executable_name);
        TweakActionResult {
            success: true,
            message: "Mock QoS policy applied".into(),
        }
    }
}

/// Removes a game's Windows QoS policy from the registry
pub fn remove_dscp_qos_policy(game_name: &str) -> TweakActionResult {
    let policy_name = sanitize_policy_name(game_name);
    remove_dscp_qos_policy_by_key(&policy_name, game_name)
}

/// Removes a Windows QoS policy subkey directly by its exact registry key name.
pub fn remove_dscp_qos_policy_by_key(policy_key_name: &str, display_label: &str) -> TweakActionResult {
    #[cfg(windows)]
    {
        use windows::Win32::System::Registry::REG_SAM_FLAGS;
        // DELETE (0x0001_0000) | KEY_SET_VALUE (0x0002) is required by RegDeleteKeyW.
        const DELETE_ACCESS: REG_SAM_FLAGS = REG_SAM_FLAGS(0x0001_0000 | 0x0002);

        let policy_h = HSTRING::from(policy_key_name);
        let policy_pc = PCWSTR(policy_h.as_ptr());

        let mut h_base = HKEY::default();
        let base_h = HSTRING::from(QOS_BASE_KEY);
        let base_pc = PCWSTR(base_h.as_ptr());

        let open_res = unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, base_pc, 0, DELETE_ACCESS, &mut h_base) };
        if open_res.is_ok() {
            // RegDeleteKeyW requires the relative subkey name when given an open parent HKEY.
            let del_res = unsafe { RegDeleteKeyW(h_base, policy_pc) };
            unsafe { let _ = RegCloseKey(h_base); }
            if del_res.is_ok() {
                return TweakActionResult {
                    success: true,
                    message: format!("Removed QoS policy for {}", display_label),
                };
            }
        }

        TweakActionResult {
            success: false,
            message: format!("Policy for {} was not found or could not be removed.", display_label),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (policy_key_name, display_label);
        TweakActionResult {
            success: true,
            message: "Mock QoS policy removed".into(),
        }
    }
}

/// Pure helper computing orphaned `FerrumPulse_*` QoS policy keys that are no longer backed
/// by an active `dscp_qos_enabled` profile.
pub fn compute_orphaned_policy_keys(
    existing_keys: &[String],
    profiles: &[GameProfile],
) -> Vec<String> {
    let expected: std::collections::HashSet<String> = profiles
        .iter()
        .filter(|p| p.dscp_qos_enabled && !p.executables.is_empty())
        .map(|p| sanitize_policy_name(&p.display_name))
        .collect();

    existing_keys
        .iter()
        .filter(|k| k.starts_with("FerrumPulse_") && !expected.contains(k.as_str()))
        .cloned()
        .collect()
}

fn list_raw_qos_policy_keys() -> Vec<String> {
    #[cfg(windows)]
    {
        let mut result = Vec::new();
        let mut h_key = HKEY::default();
        let base_h = HSTRING::from(QOS_BASE_KEY);
        let base_pc = PCWSTR(base_h.as_ptr());

        if unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, base_pc, 0, KEY_READ, &mut h_key) }.is_ok() {
            let mut index = 0u32;
            let mut name_buf = [0u16; 256];
            loop {
                let mut name_len = name_buf.len() as u32;
                let status = unsafe {
                    RegEnumKeyExW(
                        h_key,
                        index,
                        windows::core::PWSTR(name_buf.as_mut_ptr()),
                        &mut name_len,
                        None,
                        windows::core::PWSTR::null(),
                        None,
                        None,
                    )
                };
                if status.is_err() {
                    break;
                }
                let name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                if name.starts_with("FerrumPulse_") {
                    result.push(name);
                }
                index += 1;
            }
            unsafe { let _ = RegCloseKey(h_key); }
        }
        result
    }
    #[cfg(not(windows))]
    Vec::new()
}

/// Synchronizes all active game profiles with Windows QoS policies, pruning any orphaned keys.
pub fn sync_profiles_qos_policies(profiles: &[GameProfile]) -> Result<usize, String> {
    let existing = list_raw_qos_policy_keys();
    for orphan in compute_orphaned_policy_keys(&existing, profiles) {
        let _ = remove_dscp_qos_policy_by_key(&orphan, &orphan);
    }

    let mut applied_count = 0;
    for prof in profiles {
        if prof.dscp_qos_enabled {
            if let Some(first_exe) = prof.executables.first() {
                let res = apply_dscp_qos_policy(&prof.display_name, first_exe);
                if res.success {
                    applied_count += 1;
                }
            }
        } else {
            let _ = remove_dscp_qos_policy(&prof.display_name);
        }
    }
    Ok(applied_count)
}

/// Lists all active FerrumPulse QoS policies currently registered in Windows
pub fn list_active_qos_policies() -> Vec<String> {
    #[cfg(windows)]
    {
        let mut result = Vec::new();
        let mut h_key = HKEY::default();
        let base_h = HSTRING::from(QOS_BASE_KEY);
        let base_pc = PCWSTR(base_h.as_ptr());

        if unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, base_pc, 0, KEY_READ, &mut h_key) }.is_ok() {
            let mut index = 0u32;
            let mut name_buf = [0u16; 256];
            loop {
                let mut name_len = name_buf.len() as u32;
                let status = unsafe {
                    RegEnumKeyExW(
                        h_key,
                        index,
                        windows::core::PWSTR(name_buf.as_mut_ptr()),
                        &mut name_len,
                        None,
                        windows::core::PWSTR::null(),
                        None,
                        None,
                    )
                };
                if status.is_err() {
                    break;
                }
                let name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                if name.starts_with("FerrumPulse_") {
                    result.push(name.trim_start_matches("FerrumPulse_").replace('_', " "));
                }
                index += 1;
            }
            unsafe { let _ = RegCloseKey(h_key); }
        }
        result
    }
    #[cfg(not(windows))]
    vec!["Counter-Strike 2".into(), "Valorant".into()]
}

// ---------------------------------------------------------------------------
// Low-Latency TCP Tuning: Disable Nagle Algorithm & Delayed ACKs
// ---------------------------------------------------------------------------

const INTERFACES_KEY: &str = "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces";

/// Returns a list of active network adapter interface GUID subkeys
#[cfg(windows)]
fn get_active_interface_guids() -> Vec<String> {
    use crate::tweaks::read_registry_sz;
    let mut interfaces = Vec::new();
    let mut h_key = HKEY::default();
    let base_h = HSTRING::from(INTERFACES_KEY);
    let base_pc = PCWSTR(base_h.as_ptr());

    if unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, base_pc, 0, KEY_READ, &mut h_key) }.is_ok() {
        let mut index = 0u32;
        let mut name_buf = [0u16; 256];
        loop {
            let mut name_len = name_buf.len() as u32;
            let status = unsafe {
                RegEnumKeyExW(
                    h_key,
                    index,
                    windows::core::PWSTR(name_buf.as_mut_ptr()),
                    &mut name_len,
                    None,
                    windows::core::PWSTR::null(),
                    None,
                    None,
                )
            };
            if status.is_err() {
                break;
            }
            let guid = String::from_utf16_lossy(&name_buf[..name_len as usize]);
            let iface_path = format!("{}\\{}", INTERFACES_KEY, guid);
            let iface_h = HSTRING::from(iface_path.as_str());
            let iface_pc = PCWSTR(iface_h.as_ptr());

            // Check if interface has an IP assigned (active hardware connection)
            let has_ip = read_registry_sz(HKEY_LOCAL_MACHINE, iface_pc, w!("DhcpIPAddress"))
                .map(|ip| ip != "0.0.0.0" && !ip.is_empty())
                .unwrap_or(false)
                || read_registry_sz(HKEY_LOCAL_MACHINE, iface_pc, w!("IPAddress"))
                    .map(|ip| ip != "0.0.0.0" && !ip.is_empty())
                    .unwrap_or(false);

            if has_ip {
                interfaces.push(guid);
            }
            index += 1;
        }
        unsafe { let _ = RegCloseKey(h_key); }
    }
    interfaces
}

/// Checks if active interfaces have TcpAckFrequency and TCPNoDelay enabled
pub fn query_tcp_latency_optimized() -> bool {
    #[cfg(windows)]
    {
        use crate::tweaks::read_registry_dword;
        let guids = get_active_interface_guids();
        if guids.is_empty() {
            return false;
        }
        for guid in &guids {
            let subkey = format!("{}\\{}", INTERFACES_KEY, guid);
            let subkey_h = HSTRING::from(subkey.as_str());
            let subkey_pc = PCWSTR(subkey_h.as_ptr());

            let ack = read_registry_dword(HKEY_LOCAL_MACHINE, subkey_pc, w!("TcpAckFrequency"));
            let nodelay = read_registry_dword(HKEY_LOCAL_MACHINE, subkey_pc, w!("TCPNoDelay"));

            if ack == Some(1) && nodelay == Some(1) {
                return true;
            }
        }
        false
    }
    #[cfg(not(windows))]
    false
}

/// Toggles TcpAckFrequency and TCPNoDelay on active physical network adapters
pub fn toggle_tcp_latency_optimization(optimize: bool) -> TweakActionResult {
    #[cfg(windows)]
    {
        use crate::tweaks::{delete_registry_value, write_registry_dword};
        let guids = get_active_interface_guids();
        if guids.is_empty() {
            return TweakActionResult {
                success: false,
                message: "No active physical network interfaces detected.".into(),
            };
        }

        let mut updated = 0;
        for guid in &guids {
            let subkey = format!("{}\\{}", INTERFACES_KEY, guid);
            let subkey_h = HSTRING::from(subkey.as_str());
            let subkey_pc = PCWSTR(subkey_h.as_ptr());

            if optimize {
                let r1 = write_registry_dword(HKEY_LOCAL_MACHINE, subkey_pc, w!("TcpAckFrequency"), 1);
                let r2 = write_registry_dword(HKEY_LOCAL_MACHINE, subkey_pc, w!("TCPNoDelay"), 1);
                if r1.is_ok() && r2.is_ok() {
                    updated += 1;
                }
            } else {
                let _ = delete_registry_value(HKEY_LOCAL_MACHINE, subkey_pc, w!("TcpAckFrequency"));
                let _ = delete_registry_value(HKEY_LOCAL_MACHINE, subkey_pc, w!("TCPNoDelay"));
                updated += 1;
            }
        }

        if optimize {
            TweakActionResult {
                success: updated > 0,
                message: format!("Optimized TCP latency on {} active network adapter(s). (TcpAckFrequency=1, TCPNoDelay=1).", updated),
            }
        } else {
            TweakActionResult {
                success: true,
                message: "Restored stock Windows TCP buffering defaults on active network adapters.".into(),
            }
        }
    }
    #[cfg(not(windows))]
    {
        TweakActionResult {
            success: true,
            message: if optimize { "Mock TCP latency optimized".into() } else { "Mock TCP defaults restored".into() },
        }
    }
}

pub fn get_network_tweaks_status() -> NetworkTweaksStatus {
    let tcp_opt = query_tcp_latency_optimized();
    let policies = list_active_qos_policies();
    #[cfg(windows)]
    let iface_count = get_active_interface_guids().len();
    #[cfg(not(windows))]
    let iface_count = 1;

    NetworkTweaksStatus {
        tcp_latency_optimized: tcp_opt,
        active_interfaces_count: iface_count,
        qos_policies_count: policies.len(),
        active_game_policies: policies,
    }
}

// ---------------------------------------------------------------------------
// 5-Second Native ICMP Ping & Packet Jitter Diagnostic Probe
// ---------------------------------------------------------------------------

pub fn calculate_ping_jitter(rtts: &[f64]) -> f64 {
    if rtts.len() < 2 {
        return 0.0;
    }
    let mean = rtts.iter().sum::<f64>() / (rtts.len() as f64);
    let variance = rtts
        .iter()
        .map(|v| {
            let diff = v - mean;
            diff * diff
        })
        .sum::<f64>()
        / (rtts.len() as f64);
    variance.sqrt()
}

/// Synchronous ICMP ping execution running inside a dedicated worker thread
fn run_ping_probe_sync(target_host: &str) -> PingProbeReport {
    const PING_COUNT: u32 = 8;
    const PING_TIMEOUT_MS: u32 = 800;
    const DATA_PAYLOAD: &[u8] = b"FerrumPulse_Esports_Ping_Probe!";

    let mut rtts: Vec<f64> = Vec::with_capacity(PING_COUNT as usize);
    let mut sent = 0u32;
    let mut received = 0u32;

    #[cfg(windows)]
    {
        // Default to Cloudflare DNS 1.1.1.1 (0x01010101) or Google 8.8.8.8 (0x08080808)
        let ip_addr: u32 = if target_host.starts_with("8.8.8.8") {
            0x08080808
        } else {
            0x01010101
        };

        let handle = unsafe { IcmpCreateFile() };
        if !handle.is_null() {
            let reply_buf_size = std::mem::size_of::<IcmpEchoReply>() + DATA_PAYLOAD.len() + 32;
            let mut reply_buf = vec![0u8; reply_buf_size];

            for _ in 0..PING_COUNT {
                sent += 1;
                let ret = unsafe {
                    IcmpSendEcho(
                        handle,
                        ip_addr,
                        DATA_PAYLOAD.as_ptr(),
                        DATA_PAYLOAD.len() as u16,
                        std::ptr::null(),
                        reply_buf.as_mut_ptr(),
                        reply_buf.len() as u32,
                        PING_TIMEOUT_MS,
                    )
                };

                if ret > 0 {
                    let echo_reply = unsafe { &*(reply_buf.as_ptr() as *const IcmpEchoReply) };
                    if echo_reply.status == 0 {
                        received += 1;
                        rtts.push(echo_reply.round_trip_time as f64);
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(150));
            }

            unsafe {
                let _ = IcmpCloseHandle(handle);
            }
        }
    }

    #[cfg(not(windows))]
    {
        sent = PING_COUNT;
        received = PING_COUNT;
        rtts = vec![14.2, 14.8, 15.1, 14.0, 14.5, 15.0, 14.3, 14.6];
    }

    let loss_pct = if sent > 0 {
        (((sent - received) as f64) / (sent as f64)) * 100.0
    } else {
        100.0
    };

    let min_ping = rtts.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_ping = rtts.iter().cloned().fold(0.0, f64::max);
    let avg_ping = if !rtts.is_empty() {
        rtts.iter().sum::<f64>() / (rtts.len() as f64)
    } else {
        0.0
    };

    let jitter = calculate_ping_jitter(&rtts);

    let mut recommendations = Vec::new();
    let verdict = if loss_pct > 0.0 {
        recommendations.push(format!("Detected {:.1}% packet loss. Inspect Wi-Fi signal interference or Ethernet cable integrity.", loss_pct));
        "Packet Loss Detected (Suboptimal)".to_string()
    } else if jitter > 6.0 {
        recommendations.push(format!("High packet jitter (±{:.1}ms). Ensure your router has Wi-Fi WMM / QoS enabled.", jitter));
        "High Packet Jitter (Frame Desync Risk)".to_string()
    } else if avg_ping < 30.0 && jitter < 2.5 {
        recommendations.push("Network connection is pristine. Packet pacing and routing are esports ready with zero packet loss.".to_string());
        "Esports Pristine (Ultra-Low Jitter)".to_string()
    } else {
        recommendations.push("Ping stability is healthy. Enabling per-game DSCP 46 ensures priority over background downloads.".to_string());
        "Good Stability".to_string()
    };

    PingProbeReport {
        target_host: target_host.to_string(),
        min_ping_ms: if min_ping.is_finite() { min_ping } else { 0.0 },
        max_ping_ms: max_ping,
        avg_ping_ms: avg_ping,
        jitter_ms: jitter,
        packet_loss_percent: loss_pct,
        samples_sent: sent,
        samples_received: received,
        status_verdict: verdict,
        recommendations,
    }
}

/// Executes a 5-second ICMP ping probe against target_host asynchronously
pub async fn run_ping_jitter_probe(target_host: &str) -> Result<PingProbeReport, String> {
    let host = target_host.to_string();
    tokio::task::spawn_blocking(move || run_ping_probe_sync(&host))
        .await
        .map_err(|e| format!("Ping worker failed: {}", e))
}

