// src/latency.rs - On-Demand Latency & Micro-Stutter Telemetry Engine
// 100% User-Mode Win32 NTDLL & QPC Telemetry (Zero Kernel Drivers, Anti-Cheat Safe)

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

static CANCEL_FLAG: OnceLock<Arc<AtomicBool>> = OnceLock::new();

pub fn get_cancel_flag() -> Arc<AtomicBool> {
    CANCEL_FLAG.get_or_init(|| Arc::new(AtomicBool::new(false))).clone()
}

pub fn cancel_current_benchmark() {
    let flag = get_cancel_flag();
    flag.store(true, Ordering::Relaxed);
}

pub fn reset_cancel_flag() -> Arc<AtomicBool> {
    let flag = get_cancel_flag();
    flag.store(false, Ordering::Relaxed);
    flag
}

#[cfg(windows)]
#[link(name = "ntdll")]
extern "system" {
    fn NtQueryTimerResolution(
        minimum_resolution: *mut u32,
        maximum_resolution: *mut u32,
        current_resolution: *mut u32,
    ) -> i32;

    fn NtQuerySystemInformation(
        system_information_class: u32,
        system_information: *mut std::ffi::c_void,
        system_information_length: u32,
        return_length: *mut u32,
    ) -> i32;
}

#[cfg(windows)]
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct SystemProcessorPerformanceInformation {
    idle_time: i64,
    kernel_time: i64,
    user_time: i64,
    dpc_time: i64,
    interrupt_time: i64,
    interrupt_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LatencyGrade {
    GradeAPlus, // 95 - 100
    GradeA,     // 85 - 94
    GradeB,     // 70 - 84
    GradeC,     // < 70
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyBenchmarkReport {
    pub grade: LatencyGrade,
    pub grade_label: String,
    pub score: u32,
    pub peak_dpc_us: f64,
    pub avg_dpc_percent: f64,
    pub timer_jitter_us: f64,
    pub current_timer_resolution_ms: f64,
    pub min_timer_resolution_ms: f64,
    pub audio_glitches_detected: u32,
    pub total_samples: u32,
    pub elapsed_seconds: f64,
    pub primary_culprit_hint: Option<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyProgressUpdate {
    pub percent_complete: u32,
    pub seconds_remaining: u32,
    pub current_dpc_us: f64,
    pub current_jitter_us: f64,
}

/// Raw snapshot of processor execution times
#[derive(Debug, Default, Clone, Copy)]
struct ProcessorSnapshot {
    total_dpc_100ns: i64,
    _total_interrupt_100ns: i64,
    total_time_100ns: i64,
    num_processors: usize,
}

#[cfg(windows)]
fn query_processor_snapshot() -> Option<ProcessorSnapshot> {
    const SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION_CLASS: u32 = 1;
    let num_cpus = num_cpus();
    let mut buffer = vec![
        SystemProcessorPerformanceInformation::default();
        num_cpus.max(1)
    ];

    let buffer_size = (buffer.len() * std::mem::size_of::<SystemProcessorPerformanceInformation>()) as u32;
    let mut return_length: u32 = 0;

    let status = unsafe {
        NtQuerySystemInformation(
            SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION_CLASS,
            buffer.as_mut_ptr() as *mut std::ffi::c_void,
            buffer_size,
            &mut return_length,
        )
    };

    if status != 0 {
        return None;
    }

    let actual_count = (return_length as usize) / std::mem::size_of::<SystemProcessorPerformanceInformation>();
    let mut total_dpc: i64 = 0;
    let mut total_interrupt: i64 = 0;
    let mut total_time: i64 = 0;

    for info in &buffer[..actual_count] {
        total_dpc += info.dpc_time;
        total_interrupt += info.interrupt_time;
        total_time += info.kernel_time + info.user_time;
    }

    Some(ProcessorSnapshot {
        total_dpc_100ns: total_dpc,
        _total_interrupt_100ns: total_interrupt,
        total_time_100ns: total_time,
        num_processors: actual_count.max(1),
    })
}

#[cfg(not(windows))]
fn query_processor_snapshot() -> Option<ProcessorSnapshot> {
    Some(ProcessorSnapshot {
        total_dpc_100ns: 100,
        _total_interrupt_100ns: 50,
        total_time_100ns: 10_000,
        num_processors: 8,
    })
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Query current and maximum supported timer resolution via NtQueryTimerResolution
pub fn get_timer_resolutions() -> (f64, f64) {
    #[cfg(windows)]
    unsafe {
        let mut min_res: u32 = 0;
        let mut max_res: u32 = 0;
        let mut cur_res: u32 = 0;
        let status = NtQueryTimerResolution(&mut min_res, &mut max_res, &mut cur_res);
        if status == 0 {
            // Units are in 100-nanoseconds. 10,000 units = 1.0ms
            let cur_ms = (cur_res as f64) / 10_000.0;
            let min_ms = (max_res as f64) / 10_000.0; // max_res in ntdll naming is the highest resolution (e.g. 0.5ms)
            return (cur_ms, min_ms);
        }
    }
    (15.625, 0.5)
}

/// Calculate the standard deviation (jitter) in microseconds of tick deltas
pub fn calculate_jitter_us(intervals_us: &[f64]) -> f64 {
    if intervals_us.len() < 2 {
        return 0.0;
    }
    let mean = intervals_us.iter().sum::<f64>() / (intervals_us.len() as f64);
    let variance = intervals_us
        .iter()
        .map(|val| {
            let diff = val - mean;
            diff * diff
        })
        .sum::<f64>()
        / (intervals_us.len() as f64);
    variance.sqrt()
}

/// Synthesize raw metrics into a structured report with grading and recommendations
pub fn synthesize_report(
    peak_dpc_us: f64,
    avg_dpc_percent: f64,
    timer_jitter_us: f64,
    current_timer_res_ms: f64,
    min_timer_res_ms: f64,
    audio_glitches: u32,
    total_samples: u32,
    elapsed_seconds: f64,
) -> LatencyBenchmarkReport {
    let mut score: i32 = 100;
    let mut recommendations = Vec::new();
    let mut primary_culprit_hint = None;

    // 1. DPC Peak Penalty
    if peak_dpc_us > 1000.0 {
        score -= 28;
        primary_culprit_hint = Some("Critical DPC latency spike (>1000µs) detected in driver subsystem".to_string());
        recommendations.push("A device driver exceeded 1000µs execution time. Update GPU and Network adapter drivers, or disable Energy-Efficient Ethernet in Device Manager.".to_string());
    } else if peak_dpc_us > 500.0 {
        score -= 14;
        if primary_culprit_hint.is_none() {
            primary_culprit_hint = Some("Moderate DPC latency spike (500–1000µs)".to_string());
        }
        recommendations.push("Moderate DPC latency observed. Ensure GPU Power Management is set to 'Prefer Maximum Performance' or high-performance power plan.".to_string());
    } else if peak_dpc_us > 250.0 {
        score -= 5;
    }

    // 2. Average DPC Load Penalty
    if avg_dpc_percent > 4.0 {
        score -= 18;
        recommendations.push("High background interrupt load. Close background monitoring tools and peripheral RGB suites.".to_string());
    } else if avg_dpc_percent > 1.5 {
        score -= 7;
    }

    // 3. Timer Jitter Penalty
    if timer_jitter_us > 80.0 {
        score -= 16;
        recommendations.push("High platform timer jitter (>80µs). Windows kernel timer cadence is fluctuating, which causes inconsistent frame pacing.".to_string());
    } else if timer_jitter_us > 30.0 {
        score -= 8;
    }

    // 4. Timer Resolution Verification
    if current_timer_res_ms > 2.0 {
        score -= 10;
        recommendations.push("System platform timer is running at stock Windows resolution (~15.6ms). Activate 'Dynamic 1.0ms Timer' in FerrumPulse to eliminate scheduling stutter.".to_string());
    }

    // 5. Audio Glitch Penalty
    if audio_glitches > 0 {
        score -= (audio_glitches as i32 * 12).min(24);
        recommendations.push(format!("Detected {} CoreAudio buffer underruns. Check audio device bit depth or audiodg.exe priority.", audio_glitches));
    }

    let final_score = score.clamp(0, 100) as u32;
    let (grade, grade_label) = match final_score {
        95..=100 => (LatencyGrade::GradeAPlus, "A+".to_string()),
        85..=94 => (LatencyGrade::GradeA, "A".to_string()),
        70..=84 => (LatencyGrade::GradeB, "B".to_string()),
        _ => (LatencyGrade::GradeC, "C".to_string()),
    };

    if final_score >= 95 {
        recommendations.push("System latency is pristine. DPC queue and platform timer are fully stabilized for 240Hz+ esports competition.".to_string());
    }

    LatencyBenchmarkReport {
        grade,
        grade_label,
        score: final_score,
        peak_dpc_us,
        avg_dpc_percent,
        timer_jitter_us,
        current_timer_resolution_ms: current_timer_res_ms,
        min_timer_resolution_ms: min_timer_res_ms,
        audio_glitches_detected: audio_glitches,
        total_samples,
        elapsed_seconds,
        primary_culprit_hint,
        recommendations,
    }
}

/// Executes the full 15-second On-Demand Benchmark (150 samples @ 100ms)
pub async fn run_benchmark_session<F>(
    cancel_token: Arc<AtomicBool>,
    mut on_progress: F,
) -> Result<LatencyBenchmarkReport, String>
where
    F: FnMut(LatencyProgressUpdate),
{
    const TOTAL_SAMPLES: u32 = 150;
    const SAMPLE_INTERVAL_MS: u64 = 100;

    let mut dpc_deltas_us: Vec<f64> = Vec::with_capacity(TOTAL_SAMPLES as usize);
    let mut tick_deltas_us: Vec<f64> = Vec::with_capacity(TOTAL_SAMPLES as usize);
    let mut total_dpc_100ns_accum: i64 = 0;
    let mut total_time_100ns_accum: i64 = 0;

    let mut last_snapshot = query_processor_snapshot().ok_or("Failed to query processor snapshot")?;
    let mut last_tick = std::time::Instant::now();
    let start_time = std::time::Instant::now();

    // Start silent shared-mode WASAPI event-driven CoreAudio deadline probe in parallel
    let audio_probe = crate::audio::AudioGlitchProbe::start();

    for i in 1..=TOTAL_SAMPLES {
        if cancel_token.load(Ordering::Relaxed) {
            let _ = audio_probe.finish();
            return Err("Benchmark cancelled by user".to_string());
        }

        tokio::time::sleep(Duration::from_millis(SAMPLE_INTERVAL_MS)).await;

        let now_tick = std::time::Instant::now();
        let tick_elapsed_us = now_tick.duration_since(last_tick).as_micros() as f64;
        last_tick = now_tick;
        tick_deltas_us.push(tick_elapsed_us);

        if let Some(curr) = query_processor_snapshot() {
            let dpc_delta_100ns = (curr.total_dpc_100ns - last_snapshot.total_dpc_100ns).max(0);
            let time_delta_100ns = (curr.total_time_100ns - last_snapshot.total_time_100ns).max(1);

            total_dpc_100ns_accum += dpc_delta_100ns;
            total_time_100ns_accum += time_delta_100ns;

            // Convert 100ns units to microseconds: 1 microsecond = 10 100ns units
            // Spread across logical processors for single-core equivalent peak
            let dpc_delta_us = (dpc_delta_100ns as f64) / (10.0 * (curr.num_processors as f64));
            dpc_deltas_us.push(dpc_delta_us);

            last_snapshot = curr;
        }

        let seconds_remaining = ((TOTAL_SAMPLES - i) * (SAMPLE_INTERVAL_MS as u32)) / 1000;
        let percent_complete = (i * 100) / TOTAL_SAMPLES;
        let current_peak_dpc = dpc_deltas_us.iter().cloned().fold(0.0_f64, f64::max);
        let current_jitter = calculate_jitter_us(&tick_deltas_us);

        on_progress(LatencyProgressUpdate {
            percent_complete,
            seconds_remaining,
            current_dpc_us: current_peak_dpc,
            current_jitter_us: current_jitter,
        });
    }

    let (wasapi_active, wasapi_glitches) = audio_probe.finish();
    let audio_glitches = if wasapi_active {
        wasapi_glitches
    } else {
        crate::audio::count_dpc_fallback_glitches(&dpc_deltas_us)
    };

    let elapsed = start_time.elapsed().as_secs_f64();
    let peak_dpc_us = dpc_deltas_us.iter().cloned().fold(0.0_f64, f64::max);
    let avg_dpc_percent = if total_time_100ns_accum > 0 {
        ((total_dpc_100ns_accum as f64) / (total_time_100ns_accum as f64)) * 100.0
    } else {
        0.0
    };

    let timer_jitter_us = calculate_jitter_us(&tick_deltas_us);
    let (cur_res, min_res) = get_timer_resolutions();

    let report = synthesize_report(
        peak_dpc_us,
        avg_dpc_percent,
        timer_jitter_us,
        cur_res,
        min_res,
        audio_glitches,
        TOTAL_SAMPLES,
        elapsed,
    );

    Ok(report)
}
