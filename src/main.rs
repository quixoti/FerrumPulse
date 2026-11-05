// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
use windows::core::w;
#[cfg(windows)]
use windows::Win32::Foundation::{GetLastError, HWND, ERROR_ALREADY_EXISTS};
#[cfg(windows)]
use windows::Win32::System::Threading::CreateMutexW;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE};

fn main() {
    #[cfg(windows)]
    unsafe {
        let _mutex = CreateMutexW(None, true, w!("Global\\FerrumPulse_SingleInstance_Mutex"));
        if GetLastError() == ERROR_ALREADY_EXISTS {
            if let Ok(hwnd) = FindWindowW(None, w!("FerrumPulse | Gaming & PC Optimizer")) {
                if hwnd != HWND::default() {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                }
            }
            return;
        }
    }

    ferrumpulse::run();
}
