// FerrumPulse Controller: Shadcn UI Architecture & Native Win32 IPC Bridge

/**
 * Returns the native Tauri v2 invoke method if available
 */
function getNativeInvoke() {
  if (typeof window !== 'undefined') {
    if (window.__TAURI__?.core?.invoke) {
      return window.__TAURI__.core.invoke.bind(window.__TAURI__.core);
    }
    if (window.__TAURI_INTERNALS__?.invoke) {
      return window.__TAURI_INTERNALS__.invoke.bind(window.__TAURI_INTERNALS__);
    }
    if (window.__TAURI__?.invoke) {
      return window.__TAURI__.invoke.bind(window.__TAURI__);
    }
  }
  return null;
}

const isNative = () => !!getNativeInvoke();

const invoke = async (cmd, args = {}) => {
  const nativeInvoke = getNativeInvoke();
  if (nativeInvoke) {
    try {
      return await nativeInvoke(cmd, args);
    } catch (err) {
      console.error(`[FerrumPulse Native Error] ${cmd}:`, err);
      throw err;
    }
  }
  console.warn(`[FerrumPulse Mock Fallback] ${cmd}`, args);
  return mockHandler(cmd, args);
};

// UI Element Cache
const el = {
  // Navigation Tabs
  tabTriggers: document.querySelectorAll('.tabs-trigger'),
  tabContents: document.querySelectorAll('.tabs-content'),
  toast: document.getElementById('toast'),
  toastMsg: document.getElementById('toast-message'),

  // Frameless Window Controls & Search
  btnWindowMin: document.getElementById('btn-window-min'),
  btnWindowMax: document.getElementById('btn-window-max'),
  btnWindowClose: document.getElementById('btn-window-close'),
  commandSearchInput: document.getElementById('command-search-input'),

  // Header Actions & Status Indicators
  btnQuickBoost: document.getElementById('btn-quick-boost'),
  btnQuickTrim: document.getElementById('btn-quick-trim'),
  topDaemonLabel: document.getElementById('top-daemon-label'),
  topBatteryLabel: document.getElementById('top-battery-label'),
  batteryStatusDot: document.getElementById('battery-status-dot'),

  // Bottom Status Bar
  bottomDaemonStatus: document.getElementById('bottom-daemon-status'),
  bottomPowerStatus: document.getElementById('bottom-power-status'),
  bottomRamFreed: document.getElementById('bottom-ram-freed'),
  btnStatusBoost: document.getElementById('btn-status-boost'),
  btnStatusPurge: document.getElementById('btn-status-purge'),
  btnStatusHide: document.getElementById('btn-status-hide'),
  dashGuidText: document.getElementById('dash-guid-text'),

  // Summary Card 1: RAM & Live Sparkline
  ramPctText: document.getElementById('ram-pct-text'),
  ramUsedAvailText: document.getElementById('ram-used-avail-text'),
  ramProgress: document.getElementById('ram-progress'),
  ramSparkline: document.getElementById('ram-sparkline'),
  ramPressureBadge: document.getElementById('ram-pressure-badge'),

  // Summary Card 2: CPU Parking
  cpuParkingPctText: document.getElementById('cpu-parking-pct-text'),
  cpuParkingStateBadge: document.getElementById('cpu-parking-state-badge'),
  cpuProgress: document.getElementById('cpu-progress'),

  // Summary Card 3: Power Governor
  dashSchemeName: document.getElementById('dash-scheme-name'),
  btnQuickSetHigh: document.getElementById('btn-quick-set-high'),
  dashSchemeGuid: document.getElementById('dash-scheme-guid'),

  // Summary Card 4: Game Daemon
  dashActiveGameText: document.getElementById('dash-active-game-text'),
  dashThrottledCount: document.getElementById('dash-throttled-count'),
  dashSuspendedCount: document.getElementById('dash-suspended-count'),
  dashPidLabel: document.getElementById('dash-pid-label'),

  // Tab 1: Overview
  btnHeroBoost: document.getElementById('btn-hero-boost'),
  btnToggleAutostartOverview: document.getElementById('btn-toggle-autostart-overview'),

  // Tab 2: CPU & Power
  btnSetUnpark50: document.getElementById('btn-set-unpark-50'),
  btnSetUnpark100: document.getElementById('btn-set-unpark-100'),
  btnPlanBalanced: document.getElementById('btn-plan-balanced'),
  btnPlanHigh: document.getElementById('btn-plan-high'),
  btnPlanUltimate: document.getElementById('btn-plan-ultimate'),
  cpuSchemeGuid: document.getElementById('cpu-scheme-guid'),

  // Tab 3: Process Daemon
  chkDaemonBoost: document.getElementById('chk-daemon-boost'),
  chkDaemonThrottle: document.getElementById('chk-daemon-throttle'),
  gameTagsList: document.getElementById('game-tags-list'),
  btnScanGames: document.getElementById('btn-scan-games'),
  btnAddManualGame: document.getElementById('btn-add-manual-game'),
  btnImportProfiles: document.getElementById('btn-import-profiles'),
  btnExportProfiles: document.getElementById('btn-export-profiles'),
  profileEditorPanel: document.getElementById('profile-editor-panel'),
  profileEditorTitle: document.getElementById('profile-editor-title'),
  btnCancelProfileEdit: document.getElementById('btn-cancel-profile-edit'),
  btnSaveProfileEdit: document.getElementById('btn-save-profile-edit'),
  btnBrowseProfExe: document.getElementById('btn-browse-prof-exe'),
  editProfName: document.getElementById('edit-prof-name'),
  editProfExes: document.getElementById('edit-prof-exes'),
  editProfPower: document.getElementById('edit-prof-power'),
  editProfCores: document.getElementById('edit-prof-cores'),
  editProfHz: document.getElementById('edit-prof-hz'),
  editProfTimer: document.getElementById('edit-prof-timer'),
  editProfBoost: document.getElementById('edit-prof-boost'),
  editProfSuspend: document.getElementById('edit-prof-suspend'),
  editProfQos: document.getElementById('edit-prof-qos'),
  editProfSweep: document.getElementById('edit-prof-sweep'),
  discoveredShelf: document.getElementById('discovered-shelf'),
  discoveredShelfCount: document.getElementById('discovered-shelf-count'),
  discoveredGamesGrid: document.getElementById('discovered-games-grid'),
  btnCloseDiscovered: document.getElementById('btn-close-discovered'),
  profilesContainer: document.getElementById('profiles-container'),
  profilesEmptyState: document.getElementById('profiles-empty-state'),

  // Tab 4: Safe Tweaks
  btnRevertAllTweaks: document.getElementById('btn-revert-all-tweaks'),
  btnToggleDiagtrack: document.getElementById('btn-toggle-diagtrack'),
  btnRevertDiagtrack: document.getElementById('btn-revert-diagtrack'),
  btnToggleGamedvr: document.getElementById('btn-toggle-gamedvr'),
  btnRevertGamedvr: document.getElementById('btn-revert-gamedvr'),
  btnToggleNetwork: document.getElementById('btn-toggle-network'),
  btnRevertNetwork: document.getElementById('btn-revert-network'),
  btnToggleGlobalTimer: document.getElementById('btn-toggle-global-timer'),
  btnRevertGlobalTimer: document.getElementById('btn-revert-global-timer'),
  btnToggleRawMouse: document.getElementById('btn-toggle-raw-mouse'),
  btnRevertRawMouse: document.getElementById('btn-revert-raw-mouse'),
  btnToggleAutostart: document.getElementById('btn-toggle-autostart'),
  btnApplyAllScheduler: document.getElementById('btn-apply-all-scheduler'),
  btnToggleWin32Prio: document.getElementById('btn-toggle-win32-prio'),
  btnRevertWin32Prio: document.getElementById('btn-revert-win32-prio'),
  badgeRestartReq: document.getElementById('badge-restart-req'),
  btnToggleMmcssPrio: document.getElementById('btn-toggle-mmcss-prio'),
  btnRevertMmcssPrio: document.getElementById('btn-revert-mmcss-prio'),
  btnToggleSysResp: document.getElementById('btn-toggle-sys-resp'),
  btnRevertSysResp: document.getElementById('btn-revert-sys-resp'),
  btnToggleTcpLatency: document.getElementById('btn-toggle-tcp-latency'),
  btnRevertTcpLatency: document.getElementById('btn-revert-tcp-latency'),
  btnToggleAudioTuning: document.getElementById('btn-toggle-audio-tuning'),
  btnRevertAudioTuning: document.getElementById('btn-revert-audio-tuning'),
  badgeAudioCore: document.getElementById('badge-audio-core'),
  chkAutoTrimRam: document.getElementById('chk-auto-trim-ram'),
  latencyAudioGlitchesBadge: document.getElementById('latency-audio-glitches-badge'),

  // Network Ping & Jitter Diagnostic
  selPingTarget: document.getElementById('sel-ping-target'),
  btnRunPingTest: document.getElementById('btn-run-ping-test'),
  pingMetricAvg: document.getElementById('ping-metric-avg'),
  pingMetricMinmax: document.getElementById('ping-metric-minmax'),
  pingMetricJitter: document.getElementById('ping-metric-jitter'),
  pingMetricLoss: document.getElementById('ping-metric-loss'),
  pingMetricQos: document.getElementById('ping-metric-qos'),
  pingMetricPolicies: document.getElementById('ping-metric-policies'),
  pingVerdictTitle: document.getElementById('ping-verdict-title'),
  pingVerdictDesc: document.getElementById('ping-verdict-desc'),
  pingStatusBadge: document.getElementById('ping-status-badge'),

  // Tab 5: RAM & Storage
  memTotalStat: document.getElementById('mem-total-stat'),
  memAvailStat: document.getElementById('mem-avail-stat'),
  btnPurgeRam: document.getElementById('btn-purge-ram'),
  btnScanStorage: document.getElementById('btn-scan-storage'),
  btnExecClean: document.getElementById('btn-exec-clean'),
  sizeUserTemp: document.getElementById('size-user-temp'),
  sizeWinTemp: document.getElementById('size-win-temp'),
  sizeD3dCache: document.getElementById('size-d3d-cache'),
  sizeTotalCleanable: document.getElementById('size-total-cleanable'),
  chkCleanUserTemp: document.getElementById('chk-clean-user-temp'),
  chkCleanWinTemp: document.getElementById('chk-clean-win-temp'),
  chkCleanD3d: document.getElementById('chk-clean-d3d'),
  chkCleanRecycle: document.getElementById('chk-clean-recycle'),

  // Tab 6: Battery & Eco
  batProfileBadge: document.getElementById('bat-profile-badge'),
  batPctHero: document.getElementById('bat-pct-hero'),
  batAcStatus: document.getElementById('bat-ac-status'),
  batTimeRemaining: document.getElementById('bat-time-remaining'),
  batRefreshRate: document.getElementById('bat-refresh-rate'),
  btnToggleBatterySaver: document.getElementById('btn-toggle-battery-saver'),
  btnSetRefresh60: document.getElementById('btn-set-refresh-60'),
  chkAutoBatterySwitch: document.getElementById('chk-auto-battery-switch'),
  chkBatteryRefreshRate: document.getElementById('chk-battery-refresh-rate'),
  chkBatteryCoreParking: document.getElementById('chk-battery-core-parking'),
  chkBatteryTurboLimit: document.getElementById('chk-battery-turbo-limit'),

  // Tab 7: Windows 11 Gaming Feature Auditor
  auditorScoreText: document.getElementById('auditor-score-text'),
  auditorGradeBadge: document.getElementById('auditor-grade-badge'),
  auditorSummaryText: document.getElementById('auditor-summary-text'),
  auditorScorePercent: document.getElementById('auditor-score-percent'),
  auditorProgress: document.getElementById('auditor-progress'),
  btnReauditSystem: document.getElementById('btn-reaudit-system'),
  auditorActionableCount: document.getElementById('auditor-actionable-count'),
  auditorActionableList: document.getElementById('auditor-actionable-list'),
  auditorOptimizedCount: document.getElementById('auditor-optimized-count'),
  auditorOptimizedList: document.getElementById('auditor-optimized-list'),
  auditorAdvisoryCount: document.getElementById('auditor-advisory-count'),
  auditorAdvisoryList: document.getElementById('auditor-advisory-list'),

  // Tab 7b: Latency & Stutter Benchmarker
  btnRunLatencyTest: document.getElementById('btn-run-latency-test'),
  btnCancelLatencyTest: document.getElementById('btn-cancel-latency-test'),
  latencyIdleState: document.getElementById('latency-idle-state'),
  latencyRunningState: document.getElementById('latency-running-state'),
  latencyResultCard: document.getElementById('latency-result-card'),
  latencyCountdownText: document.getElementById('latency-countdown-text'),
  latencyProgressBar: document.getElementById('latency-progress-bar'),
  latencyLiveDpc: document.getElementById('latency-live-dpc'),
  latencyLiveJitter: document.getElementById('latency-live-jitter'),
  latencyGradeBadge: document.getElementById('latency-grade-badge'),
  latencyScoreBadge: document.getElementById('latency-score-badge'),
  latencyVerdictDesc: document.getElementById('latency-verdict-desc'),
  latencyStatusBadge: document.getElementById('latency-status-badge'),
  latencyMetricPeakDpc: document.getElementById('latency-metric-peak-dpc'),
  latencyMetricAvgDpc: document.getElementById('latency-metric-avg-dpc'),
  latencyMetricJitter: document.getElementById('latency-metric-jitter'),
  latencyMetricTimerRes: document.getElementById('latency-metric-timer-res'),
  latencyMetricTimerMin: document.getElementById('latency-metric-timer-min'),
  latencyRecommendationsList: document.getElementById('latency-recommendations-list'),
};

let toastTimeout = null;
let currentTweaksCache = null;
let batteryCache = null;
let totalRamFreedMb = 0;

function addFreedRam(mb) {
  if (typeof mb === 'number' && mb > 0) {
    totalRamFreedMb += mb;
    if (el.bottomRamFreed) {
      const label = el.bottomRamFreed.querySelector('span') || el.bottomRamFreed;
      label.textContent = `Freed: ${totalRamFreedMb} MB`;
    }
  }
}

// Real-Time Telemetry Sparkline Buffer
const ramHistory = [];
const MAX_SPARKLINE_POINTS = 40;

function drawRamSparkline(currentPct) {
  if (!el.ramSparkline) return;
  const canvas = el.ramSparkline;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  ramHistory.push(currentPct);
  if (ramHistory.length > MAX_SPARKLINE_POINTS) {
    ramHistory.shift();
  }

  const w = canvas.width;
  const h = canvas.height;
  ctx.clearRect(0, 0, w, h);

  if (ramHistory.length < 2) return;

  // Background midline guide
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.05)';
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(0, h * 0.5);
  ctx.lineTo(w, h * 0.5);
  ctx.stroke();

  // Compute curve points
  const step = w / (MAX_SPARKLINE_POINTS - 1);
  const points = ramHistory.map((val, idx) => {
    const y = h - 4 - ((val / 100) * (h - 8));
    const x = idx * step;
    return { x, y };
  });

  // Soft monochromatic fill area under line
  const fillGrad = ctx.createLinearGradient(0, 0, 0, h);
  fillGrad.addColorStop(0, 'rgba(255, 255, 255, 0.08)');
  fillGrad.addColorStop(1, 'rgba(255, 255, 255, 0.0)');

  ctx.beginPath();
  ctx.moveTo(points[0].x, h);
  points.forEach((p) => ctx.lineTo(p.x, p.y));
  ctx.lineTo(points[points.length - 1].x, h);
  ctx.closePath();
  ctx.fillStyle = fillGrad;
  ctx.fill();

  // Clean flat neutral white stroke
  ctx.beginPath();
  ctx.moveTo(points[0].x, points[0].y);
  for (let i = 1; i < points.length; i++) {
    ctx.lineTo(points[i].x, points[i].y);
  }
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.75)';
  ctx.lineWidth = 1.25;
  ctx.stroke();

  // Real-time leading marker
  const lastPoint = points[points.length - 1];
  ctx.beginPath();
  ctx.arc(lastPoint.x, lastPoint.y, 2.5, 0, Math.PI * 2);
  ctx.fillStyle = '#ffffff';
  ctx.fill();
}

// Toast Notification
function showToast(message) {
  if (!el.toast || !el.toastMsg) return;
  el.toastMsg.textContent = message;
  el.toast.classList.add('show');
  if (toastTimeout) clearTimeout(toastTimeout);
  toastTimeout = setTimeout(() => {
    el.toast.classList.remove('show');
  }, 2800);
}

// Shadcn Tabs Switching
function setTab(tabId) {
  const triggers = document.querySelectorAll('.tabs-trigger');
  const contents = document.querySelectorAll('.tabs-content');
  triggers.forEach((trigger) => {
    trigger.classList.toggle('active', trigger.getAttribute('data-tab') === tabId);
    trigger.blur();
  });
  contents.forEach((content) => {
    content.classList.toggle('active', content.id === tabId);
  });
}

document.querySelectorAll('.tabs-trigger').forEach((trigger) => {
  trigger.addEventListener('click', () => {
    trigger.blur();
    const tabId = trigger.getAttribute('data-tab');
    if (tabId) setTab(tabId);
  });
});

// Keyboard Shortcuts: Alt+B (Boost), Alt+T (Trim), 1-6 (Tabs), Ctrl+K (Search), Esc (Hide)
document.addEventListener('keydown', (e) => {
  if (e.altKey && (e.key === 'b' || e.key === 'B')) {
    e.preventDefault();
    triggerBoost();
  } else if (e.altKey && (e.key === 't' || e.key === 'T')) {
    e.preventDefault();
    triggerPurgeRam();
  } else if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K')) {
    e.preventDefault();
    if (el.commandSearchInput) {
      el.commandSearchInput.focus();
      el.commandSearchInput.select();
    }
  } else if (e.key === 'Escape') {
    if (el.commandSearchInput && document.activeElement === el.commandSearchInput) {
      el.commandSearchInput.value = '';
      filterSettings('');
      el.commandSearchInput.blur();
    } else {
      invoke('cmd_hide_window').catch(() => {});
    }
  } else if (e.altKey && (e.key === '7')) {
    e.preventDefault();
    setTab('tab-auditor');
  } else if (e.altKey && (e.key === '8')) {
    e.preventDefault();
    setTab('tab-settings');
  } else if (!e.ctrlKey && !e.altKey && !e.metaKey && e.key >= '1' && e.key <= '8') {
    if (document.activeElement?.tagName === 'INPUT' || document.activeElement?.tagName === 'TEXTAREA') return;
    const tabOrder = ['tab-overview', 'tab-cpu', 'tab-process', 'tab-tweaks', 'tab-storage', 'tab-battery', 'tab-auditor', 'tab-settings'];
    const idx = parseInt(e.key, 10) - 1;
    if (tabOrder[idx]) setTab(tabOrder[idx]);
  }
});

// Subsystem 1: Power & CPU
async function refreshPower() {
  try {
    const data = await invoke('cmd_get_power_status');
    if (!data) return;

    if (el.cpuParkingPctText) el.cpuParkingPctText.textContent = `${data.min_cores_ac_percent}%`;
    if (el.cpuParkingStateBadge) {
      el.cpuParkingStateBadge.textContent = data.is_unparked ? '100% Unparked' : '50% Stock';
      el.cpuParkingStateBadge.className = data.is_unparked ? 'badge badge-success' : 'badge badge-secondary';
    }
    if (el.cpuProgress) {
      el.cpuProgress.style.transform = `scaleX(${Math.min(100, Math.max(0, data.min_cores_ac_percent)) / 100})`;
    }

    if (el.dashSchemeName) el.dashSchemeName.textContent = data.active_scheme_name || 'Custom Plan';
    if (el.dashSchemeGuid) {
      const fullGuid = data.active_scheme_guid || '';
      el.dashSchemeGuid.setAttribute('data-full-guid', fullGuid);
      const shortGuid = fullGuid.length > 8 ? `${fullGuid.slice(0, 8)}...` : fullGuid;
      if (el.dashGuidText) {
        el.dashGuidText.textContent = shortGuid;
      } else {
        el.dashSchemeGuid.textContent = shortGuid;
      }
    }
    if (el.cpuSchemeGuid) el.cpuSchemeGuid.textContent = data.active_scheme_guid || '';

    // Update Bottom Status Bar Power State
    if (el.bottomPowerStatus) {
      const label = el.bottomPowerStatus.querySelector('span') || el.bottomPowerStatus;
      label.textContent = `Power: ${data.active_scheme_name || 'Balanced'}`;
    }

    // Segmented Buttons in Tab 2
    if (el.btnSetUnpark100) el.btnSetUnpark100.classList.toggle('active', data.is_unparked);
    if (el.btnSetUnpark50) el.btnSetUnpark50.classList.toggle('active', !data.is_unparked);

    const name = (data.active_scheme_name || '').toLowerCase();
    const isHigh = name.includes('high');
    const isUlt = name.includes('ultimate');
    if (el.btnPlanHigh) el.btnPlanHigh.classList.toggle('active', isHigh && !isUlt);
    if (el.btnPlanUltimate) el.btnPlanUltimate.classList.toggle('active', isUlt);
    if (el.btnPlanBalanced) el.btnPlanBalanced.classList.toggle('active', !isHigh && !isUlt);
  } catch (err) {
    console.error('Power refresh error:', err);
  }
}

// Subsystem 2: Memory & Live Sparkline
async function refreshMemory() {
  try {
    const mem = await invoke('cmd_get_memory_stats');
    if (!mem) return;
    const pct = mem.memory_load_percent;
    if (el.ramPctText) el.ramPctText.textContent = `${pct}%`;
    if (el.ramProgress) el.ramProgress.style.transform = `scaleX(${Math.min(100, Math.max(0, pct)) / 100})`;
    if (el.ramPressureBadge) {
      el.ramPressureBadge.style.display = (mem.high_pressure_warning || pct >= 88) ? 'inline-flex' : 'none';
    }

    // Draw live sparkline curve
    drawRamSparkline(pct);

    const usedGb = (mem.used_ram_mb / 1024).toFixed(1);
    const totalGb = (mem.total_ram_mb / 1024).toFixed(1);
    const availGb = (mem.available_ram_mb / 1024).toFixed(1);

    if (el.ramUsedAvailText) el.ramUsedAvailText.textContent = `${usedGb} GB / ${totalGb} GB`;
    if (el.memTotalStat) el.memTotalStat.textContent = `${totalGb} GB`;
    if (el.memAvailStat) el.memAvailStat.textContent = `${availGb} GB`;
  } catch (err) {
    console.error('Memory refresh error:', err);
  }
}

// Subsystem 3: Daemon
async function refreshDaemon() {
  try {
    const st = await invoke('cmd_get_daemon_status');
    if (!st) return;

    if (st.active_game) {
      if (el.dashActiveGameText) el.dashActiveGameText.textContent = st.active_game;
      if (el.dashPidLabel) el.dashPidLabel.textContent = st.boosted_pid ? `PID ${st.boosted_pid}` : '';
      if (el.topDaemonLabel) el.topDaemonLabel.textContent = `Pacing: ${st.active_game} (1.0ms Timer)`;
    } else {
      if (el.dashActiveGameText) el.dashActiveGameText.textContent = 'Desktop Focus';
      if (el.dashPidLabel) el.dashPidLabel.textContent = 'Standby';
      if (el.topDaemonLabel) el.topDaemonLabel.textContent = isNative() ? 'Daemon Active' : 'Web Preview';
    }
    if (el.dashThrottledCount) el.dashThrottledCount.textContent = st.throttled_count ?? 0;
    if (el.dashSuspendedCount) el.dashSuspendedCount.textContent = st.suspended_webviews_count ?? 0;

    // Update Bottom Status Bar Daemon Indicator
    if (el.bottomDaemonStatus) {
      const label = el.bottomDaemonStatus.querySelector('span.text-xs') || el.bottomDaemonStatus;
      if (st.active_game) {
        label.textContent = `Pacing: ${st.active_game} [1.0ms]`;
      } else {
        label.textContent = st.is_running ? 'Daemon: Active [1.0s]' : 'Daemon: Standby';
      }
    }
  } catch (err) {
    console.error('Daemon refresh error:', err);
  }
}

// Subsystem 4: Daemon Config
async function refreshDaemonConfig() {
  try {
    const cfg = await invoke('cmd_get_daemon_config');
    if (!cfg) return;

    if (el.chkDaemonBoost) el.chkDaemonBoost.checked = !!cfg.boost_priority;
    if (el.chkDaemonThrottle) el.chkDaemonThrottle.checked = !!cfg.throttle_background;

    if (el.gameTagsList && Array.isArray(cfg.target_games)) {
      el.gameTagsList.innerHTML = '';
      cfg.target_games.forEach((game) => {
        const badge = document.createElement('span');
        badge.className = 'font-mono text-xs px-2.5 py-1 rounded bg-neutral-900 border border-neutral-800 text-neutral-300';
        badge.textContent = game;
        el.gameTagsList.appendChild(badge);
      });
    }
  } catch (err) {
    console.error('Daemon config error:', err);
  }
}

// Subsystem 4b: Per-Game Custom Profiles
let editingProfileState = null;

async function loadSupportedRefreshRates() {
  if (!el.editProfHz) return;
  try {
    const rates = await invoke('cmd_get_supported_refresh_rates');
    const prevVal = el.editProfHz.value;
    el.editProfHz.innerHTML = '<option value="">Keep Current</option>';
    (rates || []).forEach((hz) => {
      const opt = document.createElement('option');
      opt.value = String(hz);
      opt.textContent = `${hz} Hz`;
      el.editProfHz.appendChild(opt);
    });
    if (prevVal) el.editProfHz.value = prevVal;
  } catch (err) {
    console.error('Refresh rates load error:', err);
  }
}

function openProfileEditor(prof) {
  if (!el.profileEditorPanel) return;
  editingProfileState = prof ? { ...prof } : {
    id: `prof-${Date.now()}`,
    display_name: '',
    executables: [],
    source: 'Manual',
    icon_path: null,
    power_scheme: 'high',
    core_unparking_percent: 100,
    lock_timer_resolution: true,
    target_refresh_rate: null,
    dscp_qos_enabled: true,
    suspend_launcher_webviews: true,
    boost_priority: true,
    post_game_ram_sweep: true,
  };

  if (el.profileEditorTitle) {
    el.profileEditorTitle.textContent = prof ? `Edit Profile: ${prof.display_name}` : 'Add Custom Game Profile';
  }
  if (el.editProfName) el.editProfName.value = editingProfileState.display_name || '';
  if (el.editProfExes) el.editProfExes.value = (editingProfileState.executables || []).join(', ');
  if (el.editProfPower) el.editProfPower.value = editingProfileState.power_scheme || 'high';
  if (el.editProfCores) el.editProfCores.value = String(editingProfileState.core_unparking_percent ?? 100);
  if (el.editProfHz) el.editProfHz.value = editingProfileState.target_refresh_rate ? String(editingProfileState.target_refresh_rate) : '';
  if (el.editProfTimer) el.editProfTimer.checked = !!editingProfileState.lock_timer_resolution;
  if (el.editProfBoost) el.editProfBoost.checked = editingProfileState.boost_priority !== false;
  if (el.editProfSuspend) el.editProfSuspend.checked = editingProfileState.suspend_launcher_webviews !== false;
  if (el.editProfQos) el.editProfQos.checked = editingProfileState.dscp_qos_enabled !== false;
  if (el.editProfSweep) el.editProfSweep.checked = editingProfileState.post_game_ram_sweep !== false;

  el.profileEditorPanel.style.display = 'block';
}

function closeProfileEditor() {
  editingProfileState = null;
  if (el.profileEditorPanel) el.profileEditorPanel.style.display = 'none';
}

async function refreshProfiles() {
  if (!el.profilesContainer) return;
  try {
    const store = await invoke('cmd_list_profiles');
    if (!store || !store.profiles) return;

    const profileList = Object.values(store.profiles);
    if (el.profilesEmptyState) {
      el.profilesEmptyState.style.display = profileList.length === 0 ? 'block' : 'none';
    }

    const existingCards = el.profilesContainer.querySelectorAll('.profile-row-card');
    existingCards.forEach((c) => c.remove());

    profileList.forEach((prof) => {
      const card = document.createElement('div');
      card.className = 'profile-row-card flex items-center justify-between p-3 rounded-lg border border-white/10 bg-white/[0.03]';
      card.innerHTML = `
        <div class="space-y-1">
          <div class="flex items-center gap-2 flex-wrap">
            <span class="font-medium text-white text-sm">${prof.display_name}</span>
            <span class="badge badge-outline text-[10px] uppercase font-mono">${prof.source || 'Manual'}</span>
            ${prof.lock_timer_resolution ? '<span class="badge badge-success text-[10px]">1.0ms Timer</span>' : ''}
            ${prof.target_refresh_rate ? `<span class="badge badge-outline text-[10px] text-amber-400 font-mono">${prof.target_refresh_rate}Hz</span>` : ''}
            ${prof.suspend_launcher_webviews !== false ? '<span class="badge badge-outline text-[10px] text-emerald-400">Webview Freeze ✓</span>' : ''}
            ${prof.dscp_qos_enabled !== false ? '<span class="badge badge-outline text-[10px] text-sky-400">DSCP 46 QoS ✓</span>' : ''}
            <span class="badge badge-secondary text-[10px]">${prof.core_unparking_percent}% Cores</span>
          </div>
          <div class="text-xs text-muted-foreground font-mono">
            Binaries: ${prof.executables.join(', ')} • Scheme: ${prof.power_scheme.toUpperCase()}
          </div>
        </div>
        <div class="flex items-center gap-2">
          <button class="btn btn-secondary btn-xs btn-edit-profile">Edit</button>
          <button class="btn btn-outline btn-xs btn-delete-profile" data-id="${prof.id}">Remove</button>
        </div>
      `;

      card.querySelector('.btn-edit-profile').addEventListener('click', () => {
        openProfileEditor(prof);
      });

      card.querySelector('.btn-delete-profile').addEventListener('click', async (e) => {
        const id = e.target.getAttribute('data-id');
        try {
          await invoke('cmd_delete_profile', { profileId: id });
          showToast(`Profile removed: ${prof.display_name}`);
          refreshProfiles();
          refreshTweaks();
        } catch (err) {
          showToast(`Delete error: ${err}`);
        }
      });

      el.profilesContainer.appendChild(card);
    });
  } catch (err) {
    console.error('Profiles refresh error:', err);
  }
}

// Subsystem 5: Safe Tweaks
async function refreshTweaks() {
  try {
    const tw = await invoke('cmd_get_tweaks_status');
    if (!tw) return;
    currentTweaksCache = tw;

    if (el.btnToggleDiagtrack) {
      el.btnToggleDiagtrack.textContent = tw.diagtrack_disabled ? 'Disabled ✓' : 'Disable DiagTrack';
      el.btnToggleDiagtrack.classList.toggle('active', !!tw.diagtrack_disabled);
    }
    if (el.btnToggleGamedvr) {
      el.btnToggleGamedvr.textContent = tw.gamedvr_disabled ? 'Disabled ✓' : 'Disable GameDVR';
      el.btnToggleGamedvr.classList.toggle('active', !!tw.gamedvr_disabled);
    }
    if (el.btnToggleNetwork) {
      el.btnToggleNetwork.textContent = tw.network_pacing_optimized ? 'Optimized ✓' : 'Optimize Pacing';
      el.btnToggleNetwork.classList.toggle('active', !!tw.network_pacing_optimized);
    }
    if (el.btnToggleGlobalTimer) {
      el.btnToggleGlobalTimer.textContent = tw.global_timer_enabled ? 'Enabled ✓' : 'Enable Global Timer';
      el.btnToggleGlobalTimer.classList.toggle('active', !!tw.global_timer_enabled);
    }
    if (el.btnToggleRawMouse) {
      el.btnToggleRawMouse.textContent = tw.raw_mouse_enabled ? 'Raw 1:1 ✓' : 'Enable Raw Mouse';
      el.btnToggleRawMouse.classList.toggle('active', !!tw.raw_mouse_enabled);
    }
    if (el.btnToggleWin32Prio) {
      el.btnToggleWin32Prio.textContent = tw.win32_priority_separation ? 'Optimized (0x26) ✓' : 'Optimize (0x26)';
      el.btnToggleWin32Prio.classList.toggle('active', !!tw.win32_priority_separation);
    }
    if (el.badgeRestartReq) {
      el.badgeRestartReq.style.display = tw.win32_priority_separation ? 'inline-flex' : 'none';
    }
    if (el.btnToggleMmcssPrio) {
      el.btnToggleMmcssPrio.textContent = tw.mmcss_gaming_priority ? 'High ✓' : 'Set High';
      el.btnToggleMmcssPrio.classList.toggle('active', !!tw.mmcss_gaming_priority);
    }
    if (el.btnToggleSysResp) {
      el.btnToggleSysResp.textContent = tw.system_responsiveness_optimized ? '10% Reserve ✓' : 'Set 10%';
      el.btnToggleSysResp.classList.toggle('active', !!tw.system_responsiveness_optimized);
    }

    const autostartText = tw.autostart_enabled ? 'Autostart Active ✓' : 'Enable Autostart';
    if (el.btnToggleAutostart) {
      el.btnToggleAutostart.textContent = autostartText;
      el.btnToggleAutostart.classList.toggle('active', !!tw.autostart_enabled);
    }
    if (el.btnToggleAutostartOverview) {
      el.btnToggleAutostartOverview.textContent = autostartText;
      el.btnToggleAutostartOverview.classList.toggle('active', !!tw.autostart_enabled);
    }

    try {
      const net = await invoke('cmd_get_network_tweaks_status');
      if (net && el.btnToggleTcpLatency) {
        el.btnToggleTcpLatency.textContent = net.tcp_latency_optimized ? 'TCP Optimized (ACK=1) ✓' : 'Optimize TCP';
        el.btnToggleTcpLatency.classList.toggle('active', !!net.tcp_latency_optimized);
      }
      if (net && el.pingMetricPolicies) {
        el.pingMetricPolicies.textContent = `${net.qos_policies_count} Policies Active`;
      }
    } catch (_) {}

    try {
      const audio = await invoke('cmd_get_audio_tuning_status');
      if (audio && el.btnToggleAudioTuning) {
        el.btnToggleAudioTuning.textContent = audio.enabled ? 'Audio Pinned ✓' : 'Optimize Audio';
        el.btnToggleAudioTuning.classList.toggle('active', !!audio.enabled);
      }
      if (audio && el.badgeAudioCore) {
        if (audio.enabled) {
          el.badgeAudioCore.textContent = audio.pinned_core_index !== null && audio.pinned_core_index !== undefined
            ? `Core ${audio.pinned_core_index} • High`
            : 'High Priority';
          el.badgeAudioCore.style.display = 'inline-flex';
        } else {
          el.badgeAudioCore.style.display = 'none';
        }
      }
    } catch (_) {}
  } catch (err) {
    console.error('Tweaks refresh error:', err);
  }
}

async function refreshBatteryConfig() {
  try {
    const cfg = await invoke('cmd_get_battery_config');
    if (!cfg) return;
    if (el.chkAutoBatterySwitch) el.chkAutoBatterySwitch.checked = !!cfg.auto_switch;
    if (el.chkBatteryRefreshRate) el.chkBatteryRefreshRate.checked = !!cfg.refresh_rate_saver;
    if (el.chkBatteryCoreParking) el.chkBatteryCoreParking.checked = cfg.dc_core_parking_enabled !== false;
    if (el.chkBatteryTurboLimit) el.chkBatteryTurboLimit.checked = cfg.turbo_limit_enabled !== false;
  } catch (err) {
    console.error('Battery config refresh error:', err);
  }
}

async function refreshMemoryGuardConfig() {
  try {
    const cfg = await invoke('cmd_get_memory_guard_config');
    if (!cfg) return;
    if (el.chkAutoTrimRam) el.chkAutoTrimRam.checked = !!cfg.auto_trim_at_high_pressure;
  } catch (err) {
    console.error('Memory guard config error:', err);
  }
}

// Subsystem 6: Storage
async function refreshStorage() {
  try {
    const res = await invoke('cmd_scan_storage');
    if (!res) return;

    if (el.sizeUserTemp) el.sizeUserTemp.textContent = `${res.user_temp_mb} MB`;
    if (el.sizeWinTemp) el.sizeWinTemp.textContent = `${res.windows_temp_mb} MB`;
    if (el.sizeD3dCache) el.sizeD3dCache.textContent = `${res.directx_cache_mb} MB`;
    if (el.sizeTotalCleanable) el.sizeTotalCleanable.textContent = `${res.total_cleanable_mb} MB`;
  } catch (err) {
    console.error('Storage scan error:', err);
  }
}

// Subsystem 7: Laptop & Battery Management
async function refreshBattery() {
  try {
    const bat = await invoke('cmd_get_battery_status');
    if (!bat) return;
    batteryCache = bat;

    // Header Live Pill
    if (el.topBatteryLabel) {
      const mode = bat.ac_connected ? '⚡ AC' : '🔋 DC';
      el.topBatteryLabel.textContent = `${bat.battery_percent}% ${mode}`;
    }
    if (el.batteryStatusDot) {
      el.batteryStatusDot.classList.toggle('ac-online', !!bat.ac_connected);
    }

    // Tab 6 Hero Details
    if (el.batPctHero) el.batPctHero.textContent = `${bat.battery_percent}%`;
    if (el.batAcStatus) {
      el.batAcStatus.textContent = bat.ac_connected
        ? (bat.is_charging ? 'Plugged In (Charging ⚡)' : 'Plugged In (AC Online)')
        : 'On Battery Power (DC Discharging)';
    }

    if (el.batTimeRemaining) {
      if (bat.ac_connected) {
        el.batTimeRemaining.textContent = 'Continuous (AC Connected)';
      } else if (bat.remaining_time_minutes) {
        const hrs = Math.floor(bat.remaining_time_minutes / 60);
        const mins = bat.remaining_time_minutes % 60;
        el.batTimeRemaining.textContent = `${hrs}h ${mins}m estimated`;
      } else {
        el.batTimeRemaining.textContent = 'Calculating runtime...';
      }
    }

    if (el.batRefreshRate) {
      el.batRefreshRate.textContent = `${bat.current_refresh_rate} Hz`;
    }

    if (el.batProfileBadge) {
      el.batProfileBadge.textContent = bat.eco_mode_active ? 'Battery Eco Profile Active' : 'AC Performance Profile';
      el.batProfileBadge.className = bat.eco_mode_active ? 'badge badge-secondary font-mono' : 'badge badge-outline font-mono';
    }

    if (el.btnToggleBatterySaver) {
      el.btnToggleBatterySaver.textContent = bat.eco_mode_active ? 'Restore AC Performance' : 'Engage Battery Saver';
      el.btnToggleBatterySaver.classList.toggle('active', !!bat.eco_mode_active);
    }

    // Update Bottom Status Bar Power State
    if (el.bottomPowerStatus) {
      const label = el.bottomPowerStatus.querySelector('span') || el.bottomPowerStatus;
      if (bat.has_battery) {
        const pwrIcon = bat.ac_connected ? '⚡ AC' : '🔋 DC';
        label.textContent = `Power: ${bat.battery_percent}% ${pwrIcon}`;
      } else {
        label.textContent = bat.ac_connected ? 'Power: ⚡ AC Online' : 'Power: AC';
      }
    }
  } catch (err) {
    console.error('Battery refresh error:', err);
  }
}

// Subsystem 8: Windows 11 Gaming Feature Auditor
async function refreshAuditor() {
  if (!el.auditorScoreText) return;
  try {
    const report = await invoke('cmd_audit_win11_gaming_features');
    if (!report) return;

    // 1. Overall Score & Grade Badge
    const score = report.readiness_score || 0;
    el.auditorScoreText.textContent = `${score}%`;
    if (el.auditorScorePercent) el.auditorScorePercent.textContent = `${score}%`;
    if (el.auditorProgress) el.auditorProgress.style.width = `${score}%`;

    if (el.auditorGradeBadge) {
      if (score >= 90) {
        el.auditorGradeBadge.textContent = '100% Esports Ready';
        el.auditorGradeBadge.className = 'badge badge-success font-mono';
      } else if (score >= 70) {
        el.auditorGradeBadge.textContent = 'Actionable Optimizations';
        el.auditorGradeBadge.className = 'badge badge-secondary font-mono text-amber-400';
      } else {
        el.auditorGradeBadge.textContent = 'Suboptimal Configuration';
        el.auditorGradeBadge.className = 'badge badge-outline font-mono text-rose-400';
      }
    }

    if (el.auditorSummaryText) {
      if (score >= 90) {
        el.auditorSummaryText.textContent = 'All primary Windows 11 gaming features and kernel scheduling flags are fully optimized.';
      } else {
        el.auditorSummaryText.textContent = 'Detected opportunities to improve frame pacing, flip model presentation, and hardware scheduling.';
      }
    }

    // 2. Clear lists
    if (el.auditorActionableList) el.auditorActionableList.innerHTML = '';
    if (el.auditorOptimizedList) el.auditorOptimizedList.innerHTML = '';
    if (el.auditorAdvisoryList) el.auditorAdvisoryList.innerHTML = '';

    const actionable = (report.checkpoints || []).filter((c) => c.tier === 'Actionable');
    const optimized = (report.checkpoints || []).filter((c) => c.tier === 'Optimized');
    const advisory = (report.checkpoints || []).filter((c) => c.tier === 'Advisory');

    if (el.auditorActionableCount) el.auditorActionableCount.textContent = actionable.length;
    if (el.auditorOptimizedCount) el.auditorOptimizedCount.textContent = optimized.length;
    if (el.auditorAdvisoryCount) el.auditorAdvisoryCount.textContent = advisory.length;

    // Render Actionable Tier
    actionable.forEach((item) => {
      const card = document.createElement('div');
      card.className = 'setting-row tweak-item border border-white/5 p-3 rounded-lg bg-white/[0.02]';

      const title = item.title || item.name || 'System Feature';
      const statusText = item.status_text || item.status || 'Actionable';
      const explanation = item.explanation || item.description || '';

      let actionControl = '';
      if (item.id === 'hags') {
        if (report.hags_state === 'Unsupported') {
          actionControl = `<span class="badge badge-outline text-[11px] text-muted-foreground font-mono">Unsupported by GPU</span>`;
        } else {
          const isHagsOn = report.hags_state === 'Enabled' || item.is_optimized;
          actionControl = `
            <button class="btn ${isHagsOn ? 'btn-primary' : 'btn-secondary'} btn-xs btn-toggle-hags-action" data-enabled="${isHagsOn}">
              ${isHagsOn ? 'Enabled (Reboot Req) ✓' : 'Enable HAGS'}
            </button>
          `;
        }
      } else if (item.id === 'windowed_opt' || item.id === 'windowed_flip') {
        const isFlipOn = !!item.is_optimized;
        actionControl = `
          <button class="btn ${isFlipOn ? 'btn-primary' : 'btn-secondary'} btn-xs btn-toggle-flip-action" data-enabled="${isFlipOn}">
            ${isFlipOn ? 'Flip Model Active ✓' : 'Enable Flip Model'}
          </button>
        `;
      } else if (item.id === 'game_mode') {
        const isGmOn = !!item.is_optimized;
        actionControl = `
          <button class="btn ${isGmOn ? 'btn-primary' : 'btn-secondary'} btn-xs btn-toggle-gamemode-action" data-enabled="${isGmOn}">
            ${isGmOn ? 'Game Mode Active ✓' : 'Enable Game Mode'}
          </button>
        `;
      }

      card.innerHTML = `
        <div class="max-w-xl">
          <div class="flex items-center gap-2">
            <span class="font-medium text-white text-sm">${title}</span>
            <span class="badge ${item.is_optimized ? 'badge-success' : 'badge-secondary text-amber-400'} font-mono text-[10px]">${statusText}</span>
          </div>
          <div class="text-xs text-muted-foreground mt-0.5">${explanation}</div>
        </div>
        <div class="flex items-center gap-2">
          ${actionControl}
        </div>
      `;

      const btnHags = card.querySelector('.btn-toggle-hags-action');
      if (btnHags) {
        btnHags.addEventListener('click', async () => {
          const currentlyOn = btnHags.getAttribute('data-enabled') === 'true';
          try {
            const res = await invoke('cmd_toggle_hags', { enable: !currentlyOn });
            showToast(res.message);
            refreshAuditor();
          } catch (e) {
            showToast(`HAGS error: ${e}`);
          }
        });
      }

      const btnFlip = card.querySelector('.btn-toggle-flip-action');
      if (btnFlip) {
        btnFlip.addEventListener('click', async () => {
          const currentlyOn = btnFlip.getAttribute('data-enabled') === 'true';
          try {
            const res = await invoke('cmd_toggle_windowed_optimizations', { enable: !currentlyOn });
            showToast(res.message);
            refreshAuditor();
          } catch (e) {
            showToast(`Windowed optimizations error: ${e}`);
          }
        });
      }

      const btnGm = card.querySelector('.btn-toggle-gamemode-action');
      if (btnGm) {
        btnGm.addEventListener('click', async () => {
          const currentlyOn = btnGm.getAttribute('data-enabled') === 'true';
          try {
            const res = await invoke('cmd_toggle_game_mode', { enable: !currentlyOn });
            showToast(res.message);
            refreshAuditor();
          } catch (e) {
            showToast(`Game Mode error: ${e}`);
          }
        });
      }

      el.auditorActionableList.appendChild(card);
    });

    // Render Optimized Tier
    optimized.forEach((item) => {
      const card = document.createElement('div');
      card.className = 'setting-row tweak-item border border-white/5 p-3 rounded-lg bg-white/[0.02]';
      const title = item.title || item.name || 'System Feature';
      const statusText = item.status_text || item.status || 'Verified Active';
      const explanation = item.explanation || item.description || '';

      card.innerHTML = `
        <div class="max-w-xl">
          <div class="flex items-center gap-2">
            <span class="font-medium text-white text-sm">${title}</span>
            <span class="badge badge-success font-mono text-[10px]">Optimal ✓</span>
          </div>
          <div class="text-xs text-muted-foreground mt-0.5">${explanation}</div>
        </div>
        <div>
          <span class="badge badge-outline text-[11px] text-emerald-400 font-mono">${statusText}</span>
        </div>
      `;
      el.auditorOptimizedList.appendChild(card);
    });

    // Render Advisory Tier
    advisory.forEach((item) => {
      const card = document.createElement('div');
      card.className = 'setting-row tweak-item border border-white/5 p-3 rounded-lg bg-white/[0.02]';
      const title = item.title || item.name || 'Advisory Context';
      const statusText = item.status_text || item.status || 'Advisory';
      const explanation = item.explanation || item.description || '';

      const deepLinkUri = (item.action && item.action.type === 'DeepLink')
        ? (item.action.payload?.uri || item.action.payload)
        : (item.deep_link_uri || null);

      card.innerHTML = `
        <div class="max-w-xl">
          <div class="flex items-center gap-2">
            <span class="font-medium text-white text-sm">${title}</span>
            <span class="badge badge-secondary font-mono text-[10px] text-sky-400">${statusText}</span>
          </div>
          <div class="text-xs text-muted-foreground mt-0.5">${explanation}</div>
        </div>
        <div class="flex items-center gap-2">
          ${deepLinkUri ? `
            <button class="btn btn-outline btn-xs btn-open-settings" data-uri="${deepLinkUri}">
              Open Settings ↗
            </button>
          ` : ''}
        </div>
      `;

      const btnOpen = card.querySelector('.btn-open-settings');
      if (btnOpen) {
        btnOpen.addEventListener('click', async () => {
          const uri = btnOpen.getAttribute('data-uri');
          try {
            await invoke('cmd_open_windows_settings', { uri });
          } catch (e) {
            showToast(`Settings error: ${e}`);
          }
        });
      }

      el.auditorAdvisoryList.appendChild(card);
    });
  } catch (err) {
    console.error('Auditor refresh error:', err);
  }
}

// Subsystem 8b: Real-Time Latency & Stutter Benchmarker
let isBenchmarkingLatency = false;

function setupLatencyBenchmarker() {
  if (window.__TAURI__?.event?.listen) {
    try {
      window.__TAURI__.event.listen('latency_progress', (event) => {
        const update = event.payload;
        if (!update) return;
        if (el.latencyProgressBar) el.latencyProgressBar.style.width = `${update.percent_complete}%`;
        if (el.latencyCountdownText) el.latencyCountdownText.textContent = `Sampling driver DPC queues (${update.seconds_remaining}s remaining)`;
        if (el.latencyLiveDpc) el.latencyLiveDpc.textContent = `${update.current_dpc_us.toFixed(0)} µs`;
        if (el.latencyLiveJitter) el.latencyLiveJitter.textContent = `${update.current_jitter_us.toFixed(1)} µs`;
      });
    } catch (_) {}
  }

  if (el.btnRunLatencyTest) {
    el.btnRunLatencyTest.addEventListener('click', async () => {
      if (isBenchmarkingLatency) return;
      isBenchmarkingLatency = true;

      if (el.btnRunLatencyTest) el.btnRunLatencyTest.disabled = true;
      if (el.latencyIdleState) el.latencyIdleState.style.display = 'none';
      if (el.latencyResultCard) el.latencyResultCard.style.display = 'none';
      if (el.latencyRunningState) el.latencyRunningState.style.display = 'block';
      if (el.latencyProgressBar) el.latencyProgressBar.style.width = '0%';
      if (el.latencyCountdownText) el.latencyCountdownText.textContent = 'Initializing telemetry probes (15s remaining)...';
      if (el.latencyLiveDpc) el.latencyLiveDpc.textContent = '0 µs';
      if (el.latencyLiveJitter) el.latencyLiveJitter.textContent = '0 µs';

      try {
        const report = await invoke('cmd_run_latency_benchmark');
        renderLatencyReport(report);
        showToast(`Latency Test Complete: Grade ${report.grade_label} (${report.score}/100)`);
      } catch (err) {
        showToast(`Latency test: ${err}`);
        if (el.latencyIdleState) el.latencyIdleState.style.display = 'flex';
      } finally {
        isBenchmarkingLatency = false;
        if (el.btnRunLatencyTest) el.btnRunLatencyTest.disabled = false;
        if (el.latencyRunningState) el.latencyRunningState.style.display = 'none';
      }
    });
  }

  if (el.btnCancelLatencyTest) {
    el.btnCancelLatencyTest.addEventListener('click', async () => {
      try {
        await invoke('cmd_cancel_latency_benchmark');
        showToast('Latency benchmark cancelled.');
      } catch (e) {
        console.error('Cancel error:', e);
      }
    });
  }
}

function renderLatencyReport(report) {
  if (!report || !el.latencyResultCard) return;

  if (el.latencyGradeBadge) {
    el.latencyGradeBadge.textContent = report.grade_label;
    el.latencyGradeBadge.className = `text-4xl font-extrabold font-mono tracking-tight ${
      report.grade === 'GradeAPlus' ? 'text-emerald-400' :
      report.grade === 'GradeA' ? 'text-sky-400' :
      report.grade === 'GradeB' ? 'text-amber-400' : 'text-rose-400'
    }`;
  }

  if (el.latencyScoreBadge) {
    el.latencyScoreBadge.textContent = `Score: ${report.score} / 100`;
    el.latencyScoreBadge.className = `badge font-mono ${
      report.score >= 85 ? 'badge-success' : report.score >= 70 ? 'badge-secondary text-amber-400' : 'badge-outline text-rose-400'
    }`;
  }

  if (el.latencyStatusBadge) {
    el.latencyStatusBadge.textContent =
      report.score >= 95 ? '240Hz+ Esports Ready' :
      report.score >= 85 ? 'Optimal Responsiveness' :
      report.score >= 70 ? 'Moderate Stutter Risk' : 'Micro-Stuttering Detected';
  }

  if (el.latencyAudioGlitchesBadge) {
    const g = report.audio_glitches_detected ?? 0;
    el.latencyAudioGlitchesBadge.textContent = `${g} Audio Glitch${g === 1 ? '' : 'es'}`;
    el.latencyAudioGlitchesBadge.className = `badge badge-outline text-[11px] font-mono ${
      g === 0 ? 'text-sky-400' : 'text-rose-400'
    }`;
  }

  if (el.latencyVerdictDesc) {
    el.latencyVerdictDesc.textContent = report.primary_culprit_hint || (
      report.score >= 95
        ? 'Flawless 240Hz+ esports frame pacing consistency. Driver DPC queues and platform timer cadence are pristine.'
        : 'System is responsive with minor latency headroom.'
    );
  }

  if (el.latencyMetricPeakDpc) el.latencyMetricPeakDpc.textContent = `${report.peak_dpc_us.toFixed(0)} µs`;
  if (el.latencyMetricAvgDpc) el.latencyMetricAvgDpc.textContent = `${report.avg_dpc_percent.toFixed(2)}%`;
  if (el.latencyMetricJitter) el.latencyMetricJitter.textContent = `${report.timer_jitter_us.toFixed(1)} µs`;
  if (el.latencyMetricTimerRes) el.latencyMetricTimerRes.textContent = `${report.current_timer_resolution_ms.toFixed(2)} ms`;
  if (el.latencyMetricTimerMin) el.latencyMetricTimerMin.textContent = `Min: ${report.min_timer_resolution_ms.toFixed(2)} ms`;

  if (el.latencyRecommendationsList) {
    el.latencyRecommendationsList.innerHTML = '';
    (report.recommendations || []).forEach((rec) => {
      const li = document.createElement('li');
      li.textContent = rec;
      el.latencyRecommendationsList.appendChild(li);
    });
  }

  el.latencyResultCard.style.display = 'block';
}

// Search Filtering across all tabs and settings
function filterSettings(query) {
  const q = (query || '').trim().toLowerCase();
  const rows = document.querySelectorAll('.setting-row');
  if (!q) {
    rows.forEach((r) => {
      r.style.display = '';
    });
    return;
  }

  const tabAliases = {
    over: 'tab-overview',
    dash: 'tab-overview',
    cpu: 'tab-cpu',
    power: 'tab-cpu',
    park: 'tab-cpu',
    daem: 'tab-process',
    proc: 'tab-process',
    game: 'tab-process',
    tweak: 'tab-tweaks',
    safe: 'tab-tweaks',
    mouse: 'tab-tweaks',
    timer: 'tab-tweaks',
    dvr: 'tab-tweaks',
    diag: 'tab-tweaks',
    ram: 'tab-storage',
    mem: 'tab-storage',
    clean: 'tab-storage',
    temp: 'tab-storage',
    storage: 'tab-storage',
    bat: 'tab-battery',
    eco: 'tab-battery',
    saver: 'tab-battery',
    hz: 'tab-battery',
    refresh: 'tab-battery',
    audit: 'tab-auditor',
    hags: 'tab-auditor',
    mpo: 'tab-auditor',
    flip: 'tab-auditor',
    win11: 'tab-auditor',
    health: 'tab-auditor',
    vbs: 'tab-auditor',
    hvci: 'tab-auditor',
    vrr: 'tab-auditor',
    latency: 'tab-auditor',
    dpc: 'tab-auditor',
    stutter: 'tab-auditor',
    jitter: 'tab-auditor',
    bench: 'tab-auditor',
    ping: 'tab-tweaks',
    qos: 'tab-tweaks',
    dscp: 'tab-tweaks',
    nagle: 'tab-tweaks',
    tcp: 'tab-tweaks',
    audio: 'tab-tweaks',
    audiodg: 'tab-tweaks',
    crackle: 'tab-tweaks',
    xbox: 'tab-process',
    pressure: 'tab-storage',
    sett: 'tab-settings',
    theme: 'tab-settings',
    css: 'tab-settings',
    color: 'tab-settings',
  };

  for (const [prefix, tabId] of Object.entries(tabAliases)) {
    if (q.includes(prefix)) {
      setTab(tabId);
      break;
    }
  }

  rows.forEach((row) => {
    const text = row.textContent.toLowerCase();
    row.style.display = text.includes(q) ? '' : 'none';
  });
}

// Actions
async function triggerBoost() {
  playActionChime();
  if (el.btnQuickBoost) el.btnQuickBoost.disabled = true;
  if (el.btnHeroBoost) el.btnHeroBoost.disabled = true;
  if (el.btnStatusBoost) el.btnStatusBoost.disabled = true;
  try {
    const res = await invoke('cmd_one_click_boost');
    if (res?.memory_freed_mb) addFreedRam(res.memory_freed_mb);
    showToast(`⚡ Game Boost Active: CPU 100%, ${res.power_plan}, Freed ${res.memory_freed_mb} MB RAM`);
  } catch (err) {
    showToast(`Boost error: ${err}`);
  } finally {
    if (el.btnQuickBoost) el.btnQuickBoost.disabled = false;
    if (el.btnHeroBoost) el.btnHeroBoost.disabled = false;
    if (el.btnStatusBoost) el.btnStatusBoost.disabled = false;
    await Promise.allSettled([refreshPower(), refreshMemory(), refreshTweaks(), refreshBattery()]);
  }
}

async function triggerPurgeRam() {
  playActionChime();
  const origTrimText = el.btnPurgeRam ? el.btnPurgeRam.innerHTML : '';
  if (el.btnQuickTrim) el.btnQuickTrim.disabled = true;
  if (el.btnStatusPurge) el.btnStatusPurge.disabled = true;
  if (el.btnPurgeRam) {
    el.btnPurgeRam.disabled = true;
    el.btnPurgeRam.textContent = 'Purging...';
  }
  try {
    const res = await invoke('cmd_purge_working_sets');
    if (res?.freed_mb) addFreedRam(res.freed_mb);
    showToast(`Trimmed working sets! Reclaimed ${res.freed_mb} MB across ${res.processes_trimmed} processes.`);
  } catch (err) {
    showToast(`RAM Trim error: ${err}`);
  } finally {
    if (el.btnQuickTrim) el.btnQuickTrim.disabled = false;
    if (el.btnStatusPurge) el.btnStatusPurge.disabled = false;
    if (el.btnPurgeRam) {
      el.btnPurgeRam.disabled = false;
      el.btnPurgeRam.innerHTML = origTrimText;
    }
    await refreshMemory();
  }
}

// Wire Event Listeners
if (el.btnQuickBoost) el.btnQuickBoost.addEventListener('click', triggerBoost);
if (el.btnHeroBoost) el.btnHeroBoost.addEventListener('click', triggerBoost);
if (el.btnStatusBoost) el.btnStatusBoost.addEventListener('click', triggerBoost);

if (el.btnQuickTrim) el.btnQuickTrim.addEventListener('click', triggerPurgeRam);
if (el.btnPurgeRam) el.btnPurgeRam.addEventListener('click', triggerPurgeRam);
if (el.btnStatusPurge) el.btnStatusPurge.addEventListener('click', triggerPurgeRam);

if (el.btnStatusHide) {
  el.btnStatusHide.addEventListener('click', async () => {
    try {
      await invoke('cmd_hide_window');
    } catch (e) {
      console.error(e);
    }
  });
}

// Frameless Window Control Buttons
if (el.btnWindowMin) {
  el.btnWindowMin.addEventListener('click', async () => {
    try {
      await invoke('cmd_minimize_window');
    } catch (e) {
      console.error(e);
    }
  });
}

if (el.btnWindowMax) {
  el.btnWindowMax.addEventListener('click', async () => {
    try {
      await invoke('cmd_toggle_maximize_window');
    } catch (e) {
      console.error(e);
    }
  });
}

if (el.btnWindowClose) {
  el.btnWindowClose.addEventListener('click', async () => {
    try {
      if (currentSettings.closeAction === 'exit') {
        await invoke('cmd_quit_app');
      } else {
        await invoke('cmd_hide_window');
      }
    } catch (e) {
      console.error(e);
    }
  });
}

// Native Top Bar & Navigation Bar Drag
document.querySelectorAll('.app-header, .nav-segment-bar').forEach((bar) => {
  bar.addEventListener('mousedown', (e) => {
    if (e.button === 0 && !e.target.closest('button, input, textarea, a, kbd, label, .tabs-trigger, .theme-color-pill, .window-btn')) {
      invoke('cmd_drag_window').catch(() => {});
    }
  });
});

// Command Search Bar
if (el.commandSearchInput) {
  el.commandSearchInput.addEventListener('input', (e) => {
    filterSettings(e.target.value);
  });
  el.commandSearchInput.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      el.commandSearchInput.value = '';
      filterSettings('');
      el.commandSearchInput.blur();
    } else if (e.key === 'Enter') {
      const q = el.commandSearchInput.value.trim().toLowerCase();
      if (q === 'boost') {
        triggerBoost();
      } else if (q === 'purge' || q === 'trim') {
        triggerPurgeRam();
      }
    }
  });
}

// Active Power Scheme GUID Click-to-Copy
if (el.dashSchemeGuid) {
  el.dashSchemeGuid.addEventListener('click', () => {
    const guid = el.dashSchemeGuid.getAttribute('data-full-guid') || '';
    if (guid && navigator.clipboard) {
      navigator.clipboard.writeText(guid);
      showToast('Power Scheme GUID copied to clipboard');
    }
  });
}

if (el.btnQuickSetHigh) {
  el.btnQuickSetHigh.addEventListener('click', async () => {
    try {
      await invoke('cmd_set_power_scheme', { schemeType: 'high' });
      showToast('High Performance Scheme Activated');
    } catch (e) {
      showToast(`Error: ${e}`);
    }
    refreshPower();
  });
}

if (el.btnSetUnpark100) {
  el.btnSetUnpark100.addEventListener('click', async () => {
    try {
      await invoke('cmd_set_core_unparking', { unpark: true });
      showToast('Core Unparking: 100% (Unpark All)');
    } catch (e) {
      showToast(`Error: ${e}`);
    }
    refreshPower();
  });
}

if (el.btnSetUnpark50) {
  el.btnSetUnpark50.addEventListener('click', async () => {
    try {
      await invoke('cmd_set_core_unparking', { unpark: false });
      showToast('Core Unparking: 50% (Default)');
    } catch (e) {
      showToast(`Error: ${e}`);
    }
    refreshPower();
  });
}

if (el.btnPlanHigh) {
  el.btnPlanHigh.addEventListener('click', async () => {
    try {
      await invoke('cmd_set_power_scheme', { schemeType: 'high' });
      showToast('High Performance Plan Activated');
    } catch (e) {
      showToast(`Error: ${e}`);
    }
    refreshPower();
  });
}

if (el.btnPlanUltimate) {
  el.btnPlanUltimate.addEventListener('click', async () => {
    try {
      await invoke('cmd_set_power_scheme', { schemeType: 'ultimate' });
      showToast('Ultimate Performance Plan Activated');
    } catch (e) {
      showToast(`Error: ${e}`);
    }
    refreshPower();
  });
}

if (el.btnPlanBalanced) {
  el.btnPlanBalanced.addEventListener('click', async () => {
    try {
      await invoke('cmd_set_power_scheme', { schemeType: 'balanced' });
      showToast('Balanced Plan Restored');
    } catch (e) {
      showToast(`Error: ${e}`);
    }
    refreshPower();
  });
}

// Daemon Configuration Toggles
if (el.chkDaemonBoost) {
  el.chkDaemonBoost.addEventListener('change', async () => {
    try {
      const cfg = await invoke('cmd_get_daemon_config');
      cfg.boost_priority = el.chkDaemonBoost.checked;
      await invoke('cmd_update_daemon_config', { config: cfg });
      showToast(cfg.boost_priority ? 'Process priority boost enabled' : 'Process priority boost disabled');
    } catch (e) {
      showToast(`Config error: ${e}`);
    }
  });
}

if (el.chkDaemonThrottle) {
  el.chkDaemonThrottle.addEventListener('change', async () => {
    try {
      const cfg = await invoke('cmd_get_daemon_config');
      cfg.throttle_background = el.chkDaemonThrottle.checked;
      await invoke('cmd_update_daemon_config', { config: cfg });
      showToast(cfg.throttle_background ? 'EcoQoS throttling enabled' : 'EcoQoS throttling disabled');
    } catch (e) {
      showToast(`Config error: ${e}`);
    }
  });
}

// Per-Game Profiles & Library Scanner Listeners
if (el.btnScanGames) {
  el.btnScanGames.addEventListener('click', async () => {
    el.btnScanGames.disabled = true;
    el.btnScanGames.textContent = 'Scanning Libraries...';
    try {
      const games = await invoke('cmd_scan_game_libraries');
      if (el.discoveredShelf && el.discoveredGamesGrid) {
        el.discoveredShelf.style.display = 'block';
        if (el.discoveredShelfCount) {
          el.discoveredShelfCount.textContent = `Discovered ${games.length} Installed Game${games.length === 1 ? '' : 's'}`;
        }
        el.discoveredGamesGrid.innerHTML = '';
        if (games.length === 0) {
          el.discoveredGamesGrid.innerHTML = '<div class="text-xs text-muted-foreground col-span-2 py-2">No supported Steam, Epic, or Riot library folders detected.</div>';
        } else {
          games.forEach((game) => {
            const item = document.createElement('div');
            item.className = 'flex items-center justify-between p-2 rounded bg-neutral-900/80 border border-white/5 text-xs';
            item.innerHTML = `
              <div class="truncate max-w-[180px]">
                <div class="font-medium text-white truncate">${game.name}</div>
                <div class="text-[10px] text-muted-foreground font-mono truncate">${game.executable_name}</div>
              </div>
              <button class="btn btn-primary btn-xs btn-add-discovered" data-name="${game.name}" data-exe="${game.executable_name}" data-source="${game.source}">+ Profile</button>
            `;

            item.querySelector('.btn-add-discovered').addEventListener('click', async () => {
              const newProf = {
                id: `prof-${Date.now()}`,
                display_name: game.name,
                executables: [game.executable_name],
                source: game.source,
                icon_path: null,
                power_scheme: 'high',
                core_unparking_percent: 100,
                lock_timer_resolution: true,
                target_refresh_rate: null,
                suspend_launcher_webviews: true,
                boost_priority: true,
                post_game_ram_sweep: true,
              };
              try {
                await invoke('cmd_save_profile', { profile: newProf });
                showToast(`Created performance profile for ${game.name}!`);
                refreshProfiles();
              } catch (e) {
                showToast(`Save profile error: ${e}`);
              }
            });

            el.discoveredGamesGrid.appendChild(item);
          });
        }
      }
    } catch (e) {
      showToast(`Scan error: ${e}`);
    } finally {
      el.btnScanGames.disabled = false;
      el.btnScanGames.textContent = 'Scan Libraries (Steam/Epic/Riot)';
    }
  });
}

if (el.btnCloseDiscovered) {
  el.btnCloseDiscovered.addEventListener('click', () => {
    if (el.discoveredShelf) el.discoveredShelf.style.display = 'none';
  });
}

if (el.btnAddManualGame) {
  el.btnAddManualGame.addEventListener('click', async () => {
    try {
      const picked = await invoke('cmd_pick_game_executable');
      if (picked) {
        openProfileEditor({
          id: `prof-${Date.now()}`,
          display_name: picked.display_name,
          executables: [picked.executable_name],
          source: 'Manual',
          icon_path: null,
          power_scheme: 'high',
          core_unparking_percent: 100,
          lock_timer_resolution: true,
          target_refresh_rate: null,
          dscp_qos_enabled: true,
          suspend_launcher_webviews: true,
          boost_priority: true,
          post_game_ram_sweep: true,
        });
      } else {
        openProfileEditor(null);
      }
    } catch (e) {
      openProfileEditor(null);
    }
  });
}

if (el.btnBrowseProfExe) {
  el.btnBrowseProfExe.addEventListener('click', async () => {
    try {
      const picked = await invoke('cmd_pick_game_executable');
      if (picked && el.editProfExes) {
        const cur = el.editProfExes.value.trim();
        el.editProfExes.value = cur ? `${cur}, ${picked.executable_name}` : picked.executable_name;
        if (el.editProfName && !el.editProfName.value.trim()) {
          el.editProfName.value = picked.display_name;
        }
      }
    } catch (e) {
      showToast(`File picker error: ${e}`);
    }
  });
}

if (el.btnCancelProfileEdit) {
  el.btnCancelProfileEdit.addEventListener('click', closeProfileEditor);
}

if (el.btnSaveProfileEdit) {
  el.btnSaveProfileEdit.addEventListener('click', async () => {
    const name = (el.editProfName?.value || '').trim();
    const rawExes = (el.editProfExes?.value || '')
      .split(',')
      .map((s) => s.trim())
      .filter((s) => s.length > 0);

    if (!name || rawExes.length === 0) {
      showToast('Please enter a display name and at least one executable (.exe).');
      return;
    }

    const hzVal = el.editProfHz?.value ? parseInt(el.editProfHz.value, 10) : null;
    const profileToSave = {
      id: editingProfileState?.id || `prof-${Date.now()}`,
      display_name: name,
      executables: rawExes,
      source: editingProfileState?.source || 'Manual',
      icon_path: editingProfileState?.icon_path || null,
      power_scheme: el.editProfPower?.value || 'high',
      core_unparking_percent: parseInt(el.editProfCores?.value || '100', 10),
      lock_timer_resolution: el.editProfTimer ? el.editProfTimer.checked : true,
      target_refresh_rate: Number.isFinite(hzVal) ? hzVal : null,
      dscp_qos_enabled: el.editProfQos ? el.editProfQos.checked : true,
      suspend_launcher_webviews: el.editProfSuspend ? el.editProfSuspend.checked : true,
      boost_priority: el.editProfBoost ? el.editProfBoost.checked : true,
      post_game_ram_sweep: el.editProfSweep ? el.editProfSweep.checked : true,
    };

    try {
      await invoke('cmd_save_profile', { profile: profileToSave });
      showToast(`Saved profile: ${profileToSave.display_name}`);
      closeProfileEditor();
      refreshProfiles();
      refreshTweaks();
    } catch (e) {
      showToast(`Save error: ${e}`);
    }
  });
}

if (el.btnExportProfiles) {
  el.btnExportProfiles.addEventListener('click', async () => {
    try {
      const path = await invoke('cmd_export_profiles_to_file');
      if (path) showToast(`Exported profiles to ${path}`);
    } catch (e) {
      showToast(`Export error: ${e}`);
    }
  });
}

if (el.btnImportProfiles) {
  el.btnImportProfiles.addEventListener('click', async () => {
    try {
      const count = await invoke('cmd_import_profiles_from_file');
      if (count !== null && count !== undefined) {
        showToast(`Imported ${count} profile${count === 1 ? '' : 's'}!`);
        refreshProfiles();
        refreshTweaks();
      }
    } catch (e) {
      showToast(`Import error: ${e}`);
    }
  });
}

// Tweaks Actions
if (el.btnToggleDiagtrack) {
  el.btnToggleDiagtrack.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_diagtrack', { disable: true });
      showToast(res.message);
    } catch (e) {
      showToast(`DiagTrack error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnRevertDiagtrack) {
  el.btnRevertDiagtrack.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_diagtrack', { disable: false });
      showToast(res.message);
    } catch (e) {
      showToast(`DiagTrack error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnToggleGamedvr) {
  el.btnToggleGamedvr.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_gamedvr', { disable: true });
      showToast(res.message);
    } catch (e) {
      showToast(`GameDVR error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnRevertGamedvr) {
  el.btnRevertGamedvr.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_gamedvr', { disable: false });
      showToast(res.message);
    } catch (e) {
      showToast(`GameDVR error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnToggleNetwork) {
  el.btnToggleNetwork.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_network_pacing', { optimize: true });
      showToast(res.message);
    } catch (e) {
      showToast(`Network error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnRevertNetwork) {
  el.btnRevertNetwork.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_network_pacing', { optimize: false });
      showToast(res.message);
    } catch (e) {
      showToast(`Network error: ${e}`);
    }
    refreshTweaks();
  });
}

// Tweak 4: Global Timer Resolution
if (el.btnToggleGlobalTimer) {
  el.btnToggleGlobalTimer.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_global_timer', { enable: true });
      showToast(res.message);
    } catch (e) {
      showToast(`Timer error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnRevertGlobalTimer) {
  el.btnRevertGlobalTimer.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_global_timer', { enable: false });
      showToast(res.message);
    } catch (e) {
      showToast(`Timer error: ${e}`);
    }
    refreshTweaks();
  });
}

// Tweak 5: Raw Esports Mouse Input
if (el.btnToggleRawMouse) {
  el.btnToggleRawMouse.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_raw_mouse', { enable: true });
      showToast(res.message);
    } catch (e) {
      showToast(`Mouse error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnRevertRawMouse) {
  el.btnRevertRawMouse.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_raw_mouse', { enable: false });
      showToast(res.message);
    } catch (e) {
      showToast(`Mouse error: ${e}`);
    }
    refreshTweaks();
  });
}

// Tweak 3b: Win32 Priority Separation
if (el.btnToggleWin32Prio) {
  el.btnToggleWin32Prio.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_win32_priority_separation', { optimize: true });
      showToast(res.message);
    } catch (e) {
      showToast(`Priority separation error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnRevertWin32Prio) {
  el.btnRevertWin32Prio.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_win32_priority_separation', { optimize: false });
      showToast(res.message);
    } catch (e) {
      showToast(`Priority separation error: ${e}`);
    }
    refreshTweaks();
  });
}

// Tweak 3c: MMCSS Gaming Priority
if (el.btnToggleMmcssPrio) {
  el.btnToggleMmcssPrio.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_mmcss_gaming_priority', { enable: true });
      showToast(res.message);
    } catch (e) {
      showToast(`MMCSS priority error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnRevertMmcssPrio) {
  el.btnRevertMmcssPrio.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_mmcss_gaming_priority', { enable: false });
      showToast(res.message);
    } catch (e) {
      showToast(`MMCSS priority error: ${e}`);
    }
    refreshTweaks();
  });
}

// Tweak 3d: System Responsiveness Reserve
if (el.btnToggleSysResp) {
  el.btnToggleSysResp.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_system_responsiveness', { optimize: true });
      showToast(res.message);
    } catch (e) {
      showToast(`SystemResponsiveness error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnRevertSysResp) {
  el.btnRevertSysResp.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_system_responsiveness', { optimize: false });
      showToast(res.message);
    } catch (e) {
      showToast(`SystemResponsiveness error: ${e}`);
    }
    refreshTweaks();
  });
}

// Apply All Gaming Scheduler Tweaks Preset
if (el.btnApplyAllScheduler) {
  el.btnApplyAllScheduler.addEventListener('click', async () => {
    try {
      await invoke('cmd_apply_all_scheduler_tweaks');
      showToast('All 4 Kernel & Multimedia Scheduler tweaks applied!');
    } catch (e) {
      showToast(`Scheduler tweaks error: ${e}`);
    }
    refreshTweaks();
  });
}

// Tweak 6: Windows Startup Autostart
const handleAutostartToggle = async () => {
  const isEnabled = currentTweaksCache?.autostart_enabled ?? false;
  try {
    const res = await invoke('cmd_toggle_autostart', { enable: !isEnabled });
    showToast(res.message);
  } catch (e) {
    showToast(`Autostart error: ${e}`);
  }
  refreshTweaks();
};

if (el.btnToggleAutostart) el.btnToggleAutostart.addEventListener('click', handleAutostartToggle);
if (el.btnToggleAutostartOverview) el.btnToggleAutostartOverview.addEventListener('click', handleAutostartToggle);

if (el.btnRevertAllTweaks) {
  el.btnRevertAllTweaks.addEventListener('click', async () => {
    try {
      await invoke('cmd_toggle_diagtrack', { disable: false });
      await invoke('cmd_toggle_gamedvr', { disable: false });
      await invoke('cmd_toggle_network_pacing', { optimize: false });
      await invoke('cmd_toggle_win32_priority_separation', { optimize: false });
      await invoke('cmd_toggle_mmcss_gaming_priority', { enable: false });
      await invoke('cmd_toggle_system_responsiveness', { optimize: false });
      await invoke('cmd_toggle_global_timer', { enable: false });
      await invoke('cmd_toggle_raw_mouse', { enable: false });
      await invoke('cmd_toggle_tcp_latency_optimization', { optimize: false });
      await invoke('cmd_toggle_audio_engine_tuning', { enable: false });
      showToast('All system presets restored to Windows defaults.');
    } catch (e) {
      showToast(`Revert error: ${e}`);
    }
    refreshTweaks();
  });
}

// Tweak 7: Esports Low-Latency TCP
if (el.btnToggleTcpLatency) {
  el.btnToggleTcpLatency.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_tcp_latency_optimization', { optimize: true });
      showToast(res.message);
    } catch (e) {
      showToast(`TCP latency error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnRevertTcpLatency) {
  el.btnRevertTcpLatency.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_tcp_latency_optimization', { optimize: false });
      showToast(res.message);
    } catch (e) {
      showToast(`TCP latency error: ${e}`);
    }
    refreshTweaks();
  });
}

// Tweak 8: Core Audio Engine (audiodg.exe) Pinning & Priority
if (el.btnToggleAudioTuning) {
  el.btnToggleAudioTuning.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_audio_engine_tuning', { enable: true });
      showToast(res.message);
    } catch (e) {
      showToast(`Audio tuning error: ${e}`);
    }
    refreshTweaks();
  });
}

if (el.btnRevertAudioTuning) {
  el.btnRevertAudioTuning.addEventListener('click', async () => {
    try {
      const res = await invoke('cmd_toggle_audio_engine_tuning', { enable: false });
      showToast(res.message);
    } catch (e) {
      showToast(`Audio tuning error: ${e}`);
    }
    refreshTweaks();
  });
}

// Network Ping & Packet Jitter Diagnostic
if (el.btnRunPingTest) {
  el.btnRunPingTest.addEventListener('click', async () => {
    const target = el.selPingTarget ? el.selPingTarget.value : '1.1.1.1';
    if (el.btnRunPingTest) el.btnRunPingTest.disabled = true;
    if (el.pingStatusBadge) {
      el.pingStatusBadge.textContent = 'Probing...';
      el.pingStatusBadge.className = 'badge badge-secondary text-amber-400 font-mono text-[11px]';
    }
    if (el.pingVerdictTitle) el.pingVerdictTitle.textContent = `Probing ${target}...`;
    if (el.pingVerdictDesc) el.pingVerdictDesc.textContent = 'Sending 8 ICMP packets to measure routing stability and jitter standard deviation...';

    try {
      const report = await invoke('cmd_run_ping_jitter_probe', { targetHost: target });
      if (el.pingMetricAvg) el.pingMetricAvg.textContent = `${report.avg_ping_ms.toFixed(1)} ms`;
      if (el.pingMetricMinmax) el.pingMetricMinmax.textContent = `Min: ${report.min_ping_ms.toFixed(1)} / Max: ${report.max_ping_ms.toFixed(1)}`;
      if (el.pingMetricJitter) el.pingMetricJitter.textContent = `±${report.jitter_ms.toFixed(2)} ms`;
      if (el.pingMetricLoss) el.pingMetricLoss.textContent = `${report.packet_loss_percent.toFixed(1)}%`;
      if (el.pingVerdictTitle) el.pingVerdictTitle.textContent = report.status_verdict;
      if (el.pingVerdictDesc) el.pingVerdictDesc.textContent = report.recommendations.join(' ');
      if (el.pingStatusBadge) {
        el.pingStatusBadge.textContent = report.jitter_ms < 3.0 && report.packet_loss_percent === 0 ? 'Pristine ✓' : 'Analyzed';
        el.pingStatusBadge.className = 'badge badge-success font-mono text-[11px]';
      }
      showToast(`Ping Test Complete: ${report.avg_ping_ms.toFixed(1)}ms (±${report.jitter_ms.toFixed(2)}ms jitter)`);
    } catch (err) {
      showToast(`Ping test error: ${err}`);
      if (el.pingVerdictTitle) el.pingVerdictTitle.textContent = 'Probe Failed';
      if (el.pingVerdictDesc) el.pingVerdictDesc.textContent = String(err);
    } finally {
      if (el.btnRunPingTest) el.btnRunPingTest.disabled = false;
    }
  });
}

// Battery & Eco Tab Listeners
if (el.btnToggleBatterySaver) {
  el.btnToggleBatterySaver.addEventListener('click', async () => {
    const isEco = batteryCache?.eco_mode_active ?? false;
    try {
      await invoke('cmd_set_battery_eco', { enable: !isEco });
      showToast(!isEco ? 'Battery Saver Engaged: Power Saver Plan + 10% Core Parking + 99% Max CPU' : 'AC Performance Profile Restored');
    } catch (e) {
      showToast(`Battery Saver error: ${e}`);
    }
    await Promise.allSettled([refreshBattery(), refreshPower()]);
  });
}

if (el.btnSetRefresh60) {
  el.btnSetRefresh60.addEventListener('click', async () => {
    try {
      await invoke('cmd_set_display_refresh_rate', { hz: 60 });
      showToast('Display refresh rate switched to 60Hz');
    } catch (e) {
      showToast(`Refresh rate error: ${e}`);
    }
    refreshBattery();
  });
}

// Persist all 4 Tab 6 Battery & Eco Policy Switches
const saveBatteryPolicySwitches = async () => {
  try {
    const existing = (await invoke('cmd_get_battery_config')) || {};
    const updated = {
      auto_switch: el.chkAutoBatterySwitch ? el.chkAutoBatterySwitch.checked : true,
      refresh_rate_saver: el.chkBatteryRefreshRate ? el.chkBatteryRefreshRate.checked : true,
      dc_core_parking_enabled: el.chkBatteryCoreParking ? el.chkBatteryCoreParking.checked : true,
      turbo_limit_enabled: el.chkBatteryTurboLimit ? el.chkBatteryTurboLimit.checked : true,
      saved_ac_refresh_rate: existing.saved_ac_refresh_rate ?? null,
      throttle_max_cpu_percent: existing.throttle_max_cpu_percent ?? 99,
      dc_core_parking_percent: existing.dc_core_parking_percent ?? 10,
    };
    await invoke('cmd_update_battery_config', { config: updated });
    showToast('Battery & Eco policy updated.');
  } catch (e) {
    showToast(`Battery policy error: ${e}`);
  }
};

[el.chkAutoBatterySwitch, el.chkBatteryRefreshRate, el.chkBatteryCoreParking, el.chkBatteryTurboLimit].forEach((chk) => {
  if (chk) chk.addEventListener('change', saveBatteryPolicySwitches);
});

// Opt-in >88% RAM Auto-Trim Switch
if (el.chkAutoTrimRam) {
  el.chkAutoTrimRam.addEventListener('change', async () => {
    try {
      await invoke('cmd_update_memory_guard_config', {
        config: {
          auto_trim_at_high_pressure: el.chkAutoTrimRam.checked,
          threshold_percent: 88,
        },
      });
      showToast(
        el.chkAutoTrimRam.checked
          ? 'Auto-Trim enabled at 88% RAM pressure (60s cooldown).'
          : 'Auto-Trim disabled (warning badge only).'
      );
    } catch (e) {
      showToast(`Memory guard error: ${e}`);
    }
  });
}

// Storage Actions
if (el.btnScanStorage) el.btnScanStorage.addEventListener('click', refreshStorage);

if (el.btnExecClean) {
  el.btnExecClean.addEventListener('click', async () => {
    el.btnExecClean.disabled = true;
    el.btnExecClean.textContent = 'Cleaning...';
    try {
      const res = await invoke('cmd_clean_storage', {
        cleanUserTemp: el.chkCleanUserTemp ? el.chkCleanUserTemp.checked : true,
        cleanWinTemp: el.chkCleanWinTemp ? el.chkCleanWinTemp.checked : true,
        cleanD3dCache: el.chkCleanD3d ? el.chkCleanD3d.checked : false,
        emptyRecycleBin: el.chkCleanRecycle ? el.chkCleanRecycle.checked : true,
      });
      if (res?.freed_mb) addFreedRam(res.freed_mb);
      showToast(`Clean complete! Freed ${res.freed_mb} MB (${res.files_deleted} files removed).`);
      refreshStorage();
    } catch (err) {
      showToast(`Clean error: ${err}`);
    } finally {
      el.btnExecClean.disabled = false;
      el.btnExecClean.textContent = 'Clean Targets';
    }
  });
}

// Windows 11 Gaming Feature Auditor
if (el.btnReauditSystem) {
  el.btnReauditSystem.addEventListener('click', async () => {
    showToast('Re-auditing Windows 11 gaming features...');
    await refreshAuditor();
  });
}

// Mock Fallback (used only when opened directly in a browser without Tauri)
function mockHandler(cmd, args) {
  switch (cmd) {
    case 'cmd_get_power_status':
      return { active_scheme_name: 'High Performance', active_scheme_guid: '8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c', min_cores_ac_percent: 100, is_unparked: true };
    case 'cmd_get_memory_stats':
      return { total_ram_mb: 32768, available_ram_mb: 22400, used_ram_mb: 10368, memory_load_percent: 32 };
    case 'cmd_get_daemon_status':
      return { is_running: true, active_game: null, boosted_pid: null, throttled_count: 0, suspended_webviews_count: 0 };
    case 'cmd_get_daemon_config':
      return { enabled: true, boost_priority: true, throttle_background: true, target_games: ['cs2.exe', 'valorant.exe', 'cyberpunk2077.exe', 'apex.exe', 'dota2.exe'], throttled_apps: ['chrome.exe', 'discord.exe'] };
    case 'cmd_get_tweaks_status':
      return {
        diagtrack_disabled: false,
        gamedvr_disabled: false,
        network_pacing_optimized: false,
        global_timer_enabled: true,
        raw_mouse_enabled: true,
        autostart_enabled: false,
        mmcss_gaming_priority: false,
        win32_priority_separation: false,
        system_responsiveness_optimized: false,
      };
    case 'cmd_toggle_win32_priority_separation':
      return { success: true, message: args.optimize ? 'Win32PrioritySeparation set to 0x26 (Requires Windows restart).' : 'Win32PrioritySeparation restored to 0x2.' };
    case 'cmd_toggle_mmcss_gaming_priority':
      return { success: true, message: args.enable ? 'MMCSS Tasks\\Games category set to High.' : 'MMCSS Tasks\\Games category restored to Medium.' };
    case 'cmd_toggle_system_responsiveness':
      return { success: true, message: args.optimize ? 'SystemResponsiveness set to 10%.' : 'SystemResponsiveness restored to 20%.' };
    case 'cmd_apply_all_scheduler_tweaks':
      return [{ success: true, message: 'All scheduler tweaks applied.' }];
    case 'cmd_get_battery_status':
      return {
        has_battery: true,
        is_charging: false,
        ac_connected: false,
        battery_percent: 85,
        remaining_time_minutes: 210,
        eco_mode_active: true,
        auto_switch_enabled: true,
        refresh_rate_saver_enabled: true,
        current_refresh_rate: 60,
      };
    case 'cmd_set_battery_eco':
      return { success: true };
    case 'cmd_get_battery_config':
      return {
        auto_switch: true,
        refresh_rate_saver: true,
        dc_core_parking_enabled: true,
        turbo_limit_enabled: true,
        saved_ac_refresh_rate: 144,
        throttle_max_cpu_percent: 99,
        dc_core_parking_percent: 10,
      };
    case 'cmd_update_battery_config':
      return null;
    case 'cmd_get_audio_tuning_status':
      return { enabled: false, audiodg_pid: 4120, pinned_core_index: null, is_high_priority: false };
    case 'cmd_toggle_audio_engine_tuning':
      return { success: true, message: args.enable ? 'Core Audio (audiodg.exe) pinned to Core 2 at High priority.' : 'Restored stock Windows priority and affinity on audiodg.exe.' };
    case 'cmd_get_memory_guard_config':
      return { auto_trim_at_high_pressure: false, threshold_percent: 88 };
    case 'cmd_update_memory_guard_config':
      return null;
    case 'cmd_set_display_refresh_rate':
      return { success: true };
    case 'cmd_toggle_global_timer':
      return { success: true, message: args.enable ? 'Global timer enabled.' : 'Global timer reverted.' };
    case 'cmd_toggle_raw_mouse':
      return { success: true, message: args.enable ? 'Raw 1:1 mouse input enabled.' : 'Mouse acceleration restored.' };
    case 'cmd_toggle_autostart':
      return { success: true, message: args.enable ? 'Autostart enabled.' : 'Autostart disabled.' };
    case 'cmd_scan_game_libraries':
      return [
        { id: 'steam-730', name: 'Counter-Strike 2', executable_name: 'cs2.exe', executable_path: 'C:\\Program Files (x86)\\Steam\\steamapps\\common\\Counter-Strike Global Offensive\\game\\bin\\win64\\cs2.exe', source: 'Steam', install_directory: 'C:\\Program Files (x86)\\Steam\\steamapps\\common\\Counter-Strike Global Offensive' },
        { id: 'epic-fortnite', name: 'Fortnite', executable_name: 'FortniteClient-Win64-Shipping.exe', executable_path: 'C:\\Program Files\\Epic Games\\Fortnite\\FortniteGame\\Binaries\\Win64\\FortniteClient-Win64-Shipping.exe', source: 'Epic', install_directory: 'C:\\Program Files\\Epic Games\\Fortnite' }
      ];
    case 'cmd_list_profiles':
      return {
        default_desktop: { power_scheme: 'balanced', core_unparking_percent: 50, lock_timer_resolution: false },
        profiles: {
          'demo-cs2': { id: 'demo-cs2', display_name: 'Counter-Strike 2', executables: ['cs2.exe'], source: 'Steam', power_scheme: 'high', core_unparking_percent: 100, lock_timer_resolution: true, target_refresh_rate: 240 }
        }
      };
    case 'cmd_save_profile':
    case 'cmd_delete_profile':
      return { success: true };
    case 'cmd_get_supported_refresh_rates':
      return [240, 144, 120, 60];
    case 'cmd_pick_game_executable':
      return { display_name: 'Custom Game', executable_name: 'game.exe', executable_path: 'C:\\Games\\Custom\\game.exe' };
    case 'cmd_export_profiles_to_file':
      return 'C:\\Users\\Demo\\Desktop\\ferrumpulse-profiles.json';
    case 'cmd_import_profiles_from_file':
      return 1;
    case 'cmd_quit_app':
      return null;
    case 'cmd_audit_win11_gaming_features':
      return {
        readiness_score: 85,
        hags_state: 'Enabled',
        checkpoints: [
          {
            id: 'hags',
            title: 'Hardware-Accelerated GPU Scheduling (HAGS)',
            explanation: 'Offloads high-frequency scheduling tasks to a dedicated GPU scheduling processor.',
            tier: 'Actionable',
            status_text: 'Enabled (WDDM 2.7+ Active)',
            is_optimized: true,
            action: { type: 'Toggle', payload: { enable: false } }
          },
          {
            id: 'windowed_opt',
            title: 'Optimizations for Windowed Games',
            explanation: 'Upgrades legacy blt presentation to modern DXGI Flip Model, reducing input latency.',
            tier: 'Actionable',
            status_text: 'Enabled (Modern Flip Model Upgrade)',
            is_optimized: true,
            action: { type: 'Toggle', payload: { enable: false } }
          },
          {
            id: 'game_mode',
            title: 'Windows Game Mode',
            explanation: 'Prioritizes game thread execution and silences background Windows updates during gameplay.',
            tier: 'Actionable',
            status_text: 'Enabled',
            is_optimized: true,
            action: { type: 'Toggle', payload: { enable: false } }
          },
          {
            id: 'mpo',
            title: 'Multi-Plane Overlay (MPO)',
            explanation: 'Enables direct GPU hardware composition without Desktop Window Manager (DWM) frame copying.',
            tier: 'Optimized',
            status_text: 'Verified Active (Hardware Planes Supported)',
            is_optimized: true,
            action: { type: 'None' }
          },
          {
            id: 'vrr',
            title: 'Variable Refresh Rate (VRR / G-Sync)',
            explanation: 'Synchronizes display refresh rate to GPU render cycles.',
            tier: 'Advisory',
            status_text: 'Hardware Sync Managed by Windows',
            is_optimized: true,
            action: { type: 'DeepLink', payload: { uri: 'ms-settings:display-advanced' } }
          },
          {
            id: 'hvci',
            title: 'Virtualization-Based Security (VBS / HVCI)',
            explanation: 'Hypervisor-enforced memory integrity. When disabled, eliminates virtualization-induced frame drops.',
            tier: 'Advisory',
            status_text: 'Security Review',
            is_optimized: true,
            action: { type: 'DeepLink', payload: { uri: 'ms-settings:windowsdefender' } }
          }
        ]
      };
    case 'cmd_toggle_hags':
      return { success: true, message: args.enable ? 'HAGS enabled (Requires system reboot).' : 'HAGS disabled.' };
    case 'cmd_toggle_windowed_optimizations':
      return { success: true, message: args.enable ? 'Optimizations for windowed games enabled.' : 'Optimizations for windowed games disabled.' };
    case 'cmd_toggle_game_mode':
      return { success: true, message: args.enable ? 'Windows Game Mode enabled.' : 'Windows Game Mode disabled.' };
    case 'cmd_open_windows_settings':
      return { success: true, message: `Opened ${args.uri}` };
    case 'cmd_run_latency_benchmark':
      return {
        grade: 'GradeAPlus',
        grade_label: 'A+',
        score: 97,
        peak_dpc_us: 142.5,
        avg_dpc_percent: 0.38,
        timer_jitter_us: 11.8,
        current_timer_resolution_ms: 1.0,
        min_timer_resolution_ms: 0.5,
        audio_glitches_detected: 0,
        total_samples: 150,
        elapsed_seconds: 15.0,
        primary_culprit_hint: null,
        recommendations: [
          'System latency is pristine. DPC queue and platform timer are fully stabilized for 240Hz+ esports competition.'
        ]
      };
    case 'cmd_cancel_latency_benchmark':
      return true;
    case 'cmd_get_network_tweaks_status':
      return { tcp_latency_optimized: false, active_interfaces_count: 2, qos_policies_count: 1, active_game_policies: ['Counter-Strike 2'] };
    case 'cmd_toggle_tcp_latency_optimization':
      return { success: true, message: args.optimize ? 'Optimized TCP latency on active network adapters (TcpAckFrequency=1, TCPNoDelay=1).' : 'Restored stock Windows TCP buffering defaults.' };
    case 'cmd_sync_all_qos_policies':
      return 1;
    case 'cmd_run_ping_jitter_probe':
      return {
        target_host: args.targetHost || '1.1.1.1',
        min_ping_ms: 11.2,
        max_ping_ms: 13.8,
        avg_ping_ms: 12.4,
        jitter_ms: 0.85,
        packet_loss_percent: 0.0,
        samples_sent: 8,
        samples_received: 8,
        status_verdict: 'Esports Pristine (Ultra-Low Jitter)',
        recommendations: ['Network connection is pristine. Packet pacing and routing are esports ready with zero packet loss.']
      };
    case 'cmd_scan_storage':
      return { user_temp_mb: 1240.5, windows_temp_mb: 340.2, directx_cache_mb: 850.0, total_cleanable_mb: 2430.7 };
    case 'cmd_purge_working_sets':
      return { freed_mb: 1420, processes_trimmed: 48, before_used_mb: 10368, after_used_mb: 8948, available_mb: 23820 };
    case 'cmd_one_click_boost':
      return { unparked: true, power_plan: 'High Performance', network_optimized: true, memory_freed_mb: 1420 };
    case 'cmd_minimize_window':
    case 'cmd_toggle_maximize_window':
    case 'cmd_hide_window':
    case 'cmd_drag_window':
      return { success: true };
    default:
      return { success: true, message: 'Action executed' };
  }
}

// ==========================================================================
// Subsystem 7: Settings & Deep Customization Engine
// ==========================================================================

const DEFAULT_SETTINGS = {
  bgOpacity: 55,
  cardOpacity: 4,
  accentColor: 'white',
  cornerStyle: 'rounded',
  uiScale: '1',
  closeAction: 'tray',
  telemetryInterval: 1200,
  audioChime: true,
};

let currentSettings = { ...DEFAULT_SETTINGS };
let telemetryTimer = null;
let customCssDebounceTimer = null;

const COLOR_MAP = {
  white: { hex: '#ffffff', glow: 'rgba(255, 255, 255, 0.14)', text: '#09090b' },
  cyan: { hex: '#38bdf8', glow: 'rgba(56, 189, 248, 0.35)', text: '#09090b' },
  emerald: { hex: '#10b981', glow: 'rgba(16, 185, 129, 0.35)', text: '#ffffff' },
  amber: { hex: '#f59e0b', glow: 'rgba(245, 158, 11, 0.35)', text: '#09090b' },
  purple: { hex: '#a855f7', glow: 'rgba(168, 85, 247, 0.35)', text: '#ffffff' },
};

const CSS_SNIPPETS = {
  oled: `:root {
  --app-bg-opacity: 0.98;
  --card-bg-opacity: 0.02;
  --card-border-opacity: 0.07;
}
.app-layout {
  background: #000000 !important;
}`,
  cyberpunk: `:root {
  --accent-color: #38bdf8;
  --accent-glow: rgba(56, 189, 248, 0.4);
  --card-bg-opacity: 0.07;
  --card-border-opacity: 0.18;
}
.glass-card {
  box-shadow: 0 0 16px rgba(56, 189, 248, 0.12);
  border-color: rgba(56, 189, 248, 0.25) !important;
}
.tabs-trigger.active {
  border-color: #38bdf8 !important;
  box-shadow: 0 0 10px rgba(56, 189, 248, 0.3);
}`,
  frosted: `:root {
  --app-bg-opacity: 0.40;
  --card-bg-opacity: 0.05;
  --card-border-opacity: 0.12;
  --win-radius: 16px;
  --card-radius: 10px;
}
.glass-card {
  backdrop-filter: blur(12px);
}`
};

function playActionChime() {
  if (!currentSettings.audioChime) return;
  try {
    const AudioContext = window.AudioContext || window.webkitAudioContext;
    if (!AudioContext) return;
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = 'sine';
    osc.frequency.setValueAtTime(880, ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(1760, ctx.currentTime + 0.05);

    gain.gain.setValueAtTime(0.06, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.07);

    osc.connect(gain);
    gain.connect(ctx.destination);

    osc.start();
    osc.stop(ctx.currentTime + 0.07);
  } catch (_) {}
}

function loadSavedSettings() {
  try {
    const saved = localStorage.getItem('ferrumpulse_settings');
    if (saved) {
      currentSettings = { ...DEFAULT_SETTINGS, ...JSON.parse(saved) };
    }
  } catch (_) {}

  // Apply visual settings to DOM
  applyVisualSettings();

  // Load saved custom CSS
  try {
    const savedCss = localStorage.getItem('ferrumpulse_custom_css') || '';
    const txtEditor = document.getElementById('txt-custom-css');
    if (txtEditor) txtEditor.value = savedCss;
    applyCustomCss(savedCss);
  } catch (_) {}
}

function saveSettings() {
  try {
    localStorage.setItem('ferrumpulse_settings', JSON.stringify(currentSettings));
  } catch (_) {}
}

function applyVisualSettings() {
  const root = document.documentElement;

  // 1. Background Opacity
  root.style.setProperty('--app-bg-opacity', (currentSettings.bgOpacity / 100).toFixed(2));
  const sliderBg = document.getElementById('setting-bg-opacity');
  const valBg = document.getElementById('val-bg-opacity');
  if (sliderBg) sliderBg.value = currentSettings.bgOpacity;
  if (valBg) valBg.textContent = `${currentSettings.bgOpacity}%`;

  // 2. Card Glass Opacity
  root.style.setProperty('--card-bg-opacity', (currentSettings.cardOpacity / 100).toFixed(3));
  const sliderCard = document.getElementById('setting-card-opacity');
  const valCard = document.getElementById('val-card-opacity');
  if (sliderCard) sliderCard.value = currentSettings.cardOpacity;
  if (valCard) valCard.textContent = `${currentSettings.cardOpacity}%`;

  // 3. Accent Theme
  const c = COLOR_MAP[currentSettings.accentColor] || COLOR_MAP.white;
  root.style.setProperty('--accent-color', c.hex);
  root.style.setProperty('--accent-glow', c.glow);
  root.style.setProperty('--accent-text', c.text);
  document.querySelectorAll('.theme-color-pill').forEach((pill) => {
    pill.classList.toggle('active', pill.dataset.color === currentSettings.accentColor);
  });

  // 4. Corner Radius
  const isRounded = currentSettings.cornerStyle === 'rounded';
  root.style.setProperty('--win-radius', isRounded ? '12px' : '0px');
  root.style.setProperty('--card-radius', isRounded ? '8px' : '2px');
  const btnCornerR = document.getElementById('btn-corner-rounded');
  const btnCornerS = document.getElementById('btn-corner-sharp');
  if (btnCornerR) btnCornerR.classList.toggle('active', isRounded);
  if (btnCornerS) btnCornerS.classList.toggle('active', !isRounded);

  // 5. UI Scale
  root.style.setProperty('--ui-scale', currentSettings.uiScale);
  const scaleMap = { '0.9': 'btn-scale-compact', '1': 'btn-scale-default', '1.1': 'btn-scale-roomy' };
  ['btn-scale-compact', 'btn-scale-default', 'btn-scale-roomy'].forEach((id) => {
    const btn = document.getElementById(id);
    if (btn) btn.classList.toggle('active', scaleMap[currentSettings.uiScale] === id);
  });

  // 6. Close Action
  const btnCloseTray = document.getElementById('btn-close-tray');
  const btnCloseExit = document.getElementById('btn-close-exit');
  if (btnCloseTray) btnCloseTray.classList.toggle('active', currentSettings.closeAction === 'tray');
  if (btnCloseExit) btnCloseExit.classList.toggle('active', currentSettings.closeAction === 'exit');

  // 7. Telemetry Polling Rate
  const pollMap = { 800: 'btn-poll-fast', 1200: 'btn-poll-normal', 3000: 'btn-poll-eco' };
  ['btn-poll-fast', 'btn-poll-normal', 'btn-poll-eco'].forEach((id) => {
    const btn = document.getElementById(id);
    if (btn) btn.classList.toggle('active', pollMap[currentSettings.telemetryInterval] === id);
  });

  // 8. Audio Chime Switch
  const chkAudio = document.getElementById('chk-setting-audio');
  if (chkAudio) chkAudio.checked = currentSettings.audioChime;
}

function applyCustomCss(cssText) {
  const styleEl = document.getElementById('user-custom-css');
  if (styleEl) styleEl.textContent = cssText;
  const statusEl = document.getElementById('custom-css-status');
  if (statusEl) {
    const lineCount = cssText.trim() ? cssText.trim().split('\n').length : 0;
    statusEl.textContent = `${lineCount} line${lineCount === 1 ? '' : 's'} active • Live Sync Active`;
  }
}

function setupSettingsListeners() {
  // Background Opacity
  const sliderBg = document.getElementById('setting-bg-opacity');
  if (sliderBg) {
    sliderBg.addEventListener('input', (e) => {
      currentSettings.bgOpacity = parseInt(e.target.value, 10);
      applyVisualSettings();
      saveSettings();
    });
  }

  // Card Opacity
  const sliderCard = document.getElementById('setting-card-opacity');
  if (sliderCard) {
    sliderCard.addEventListener('input', (e) => {
      currentSettings.cardOpacity = parseInt(e.target.value, 10);
      applyVisualSettings();
      saveSettings();
    });
  }

  // Accent Color Pills
  document.querySelectorAll('.theme-color-pill').forEach((pill) => {
    pill.addEventListener('click', () => {
      currentSettings.accentColor = pill.dataset.color || 'white';
      applyVisualSettings();
      saveSettings();
    });
  });

  // Corner Geometry
  const btnCornerR = document.getElementById('btn-corner-rounded');
  const btnCornerS = document.getElementById('btn-corner-sharp');
  if (btnCornerR) {
    btnCornerR.addEventListener('click', () => {
      currentSettings.cornerStyle = 'rounded';
      applyVisualSettings();
      saveSettings();
    });
  }
  if (btnCornerS) {
    btnCornerS.addEventListener('click', () => {
      currentSettings.cornerStyle = 'sharp';
      applyVisualSettings();
      saveSettings();
    });
  }

  // UI Scale
  const scales = [
    { id: 'btn-scale-compact', val: '0.9' },
    { id: 'btn-scale-default', val: '1' },
    { id: 'btn-scale-roomy', val: '1.1' },
  ];
  scales.forEach(({ id, val }) => {
    const btn = document.getElementById(id);
    if (btn) {
      btn.addEventListener('click', () => {
        currentSettings.uiScale = val;
        applyVisualSettings();
        saveSettings();
      });
    }
  });

  // Close Action
  const btnCloseTray = document.getElementById('btn-close-tray');
  const btnCloseExit = document.getElementById('btn-close-exit');
  if (btnCloseTray) {
    btnCloseTray.addEventListener('click', () => {
      currentSettings.closeAction = 'tray';
      applyVisualSettings();
      saveSettings();
    });
  }
  if (btnCloseExit) {
    btnCloseExit.addEventListener('click', () => {
      currentSettings.closeAction = 'exit';
      applyVisualSettings();
      saveSettings();
    });
  }

  // Telemetry Polling Rate
  const pollRates = [
    { id: 'btn-poll-fast', val: 800 },
    { id: 'btn-poll-normal', val: 1200 },
    { id: 'btn-poll-eco', val: 3000 },
  ];
  pollRates.forEach(({ id, val }) => {
    const btn = document.getElementById(id);
    if (btn) {
      btn.addEventListener('click', () => {
        currentSettings.telemetryInterval = val;
        applyVisualSettings();
        saveSettings();
        restartTelemetryLoop();
      });
    }
  });

  // Audio Chime Toggle
  const chkAudio = document.getElementById('chk-setting-audio');
  if (chkAudio) {
    chkAudio.addEventListener('change', (e) => {
      currentSettings.audioChime = e.target.checked;
      saveSettings();
      if (currentSettings.audioChime) playActionChime();
    });
  }

  // Autostart in Settings
  const btnAutostart = document.getElementById('btn-settings-autostart');
  if (btnAutostart) {
    btnAutostart.addEventListener('click', async () => {
      try {
        const res = await invoke('cmd_toggle_autostart', { enable: true });
        showToast(res?.message || 'Autostart toggled');
      } catch (err) {
        showToast(`Autostart error: ${err}`);
      }
    });
  }

  // Custom CSS Editor Debounced Input
  const txtEditor = document.getElementById('txt-custom-css');
  if (txtEditor) {
    txtEditor.addEventListener('input', (e) => {
      clearTimeout(customCssDebounceTimer);
      customCssDebounceTimer = setTimeout(() => {
        const val = e.target.value;
        applyCustomCss(val);
        localStorage.setItem('ferrumpulse_custom_css', val);
      }, 200);
    });
  }

  // Snippets
  const snippets = [
    { id: 'btn-snippet-oled', key: 'oled' },
    { id: 'btn-snippet-cyberpunk', key: 'cyberpunk' },
    { id: 'btn-snippet-frosted', key: 'frosted' },
  ];
  snippets.forEach(({ id, key }) => {
    const btn = document.getElementById(id);
    if (btn) {
      btn.addEventListener('click', () => {
        const txt = CSS_SNIPPETS[key];
        if (txtEditor) {
          txtEditor.value = txt;
          applyCustomCss(txt);
          localStorage.setItem('ferrumpulse_custom_css', txt);
          showToast(`Applied ${key.toUpperCase()} Snippet`);
        }
      });
    }
  });

  const btnClearCss = document.getElementById('btn-snippet-clear');
  if (btnClearCss) {
    btnClearCss.addEventListener('click', () => {
      if (txtEditor) txtEditor.value = '';
      applyCustomCss('');
      localStorage.removeItem('ferrumpulse_custom_css');
      showToast('Custom CSS Cleared');
    });
  }

  // Factory Reset
  const resetHandler = () => {
    currentSettings = { ...DEFAULT_SETTINGS };
    saveSettings();
    applyVisualSettings();
    if (txtEditor) txtEditor.value = '';
    applyCustomCss('');
    localStorage.removeItem('ferrumpulse_custom_css');
    restartTelemetryLoop();
    showToast('All Settings & Styles Restored to Defaults');
  };

  const btnReset = document.getElementById('btn-reset-settings');
  if (btnReset) btnReset.addEventListener('click', resetHandler);
  const btnResetAll = document.getElementById('btn-reset-all-customizations');
  if (btnResetAll) btnResetAll.addEventListener('click', resetHandler);
}

function restartTelemetryLoop() {
  if (telemetryTimer) clearInterval(telemetryTimer);
  telemetryTimer = setInterval(async () => {
    try { await refreshMemory(); } catch (_) {}
    try { await refreshDaemon(); } catch (_) {}
    try { await refreshBattery(); } catch (_) {}
  }, currentSettings.telemetryInterval);
}

// Initialization
async function init() {
  console.log('[FerrumPulse] Initializing UI. Native IPC Available:', isNative());

  // Bootstrap settings & live styling first
  loadSavedSettings();
  setupSettingsListeners();
  setupLatencyBenchmarker();

  // Settled initial fetch so a single delay never blocks the rest of the app
  await Promise.allSettled([
    refreshPower(),
    refreshMemory(),
    refreshDaemon(),
    refreshDaemonConfig(),
    loadSupportedRefreshRates(),
    refreshProfiles(),
    refreshTweaks(),
    refreshStorage(),
    refreshBattery(),
    refreshBatteryConfig(),
    refreshMemoryGuardConfig(),
    refreshAuditor(),
  ]);

  // Dynamic ambient refresh loop
  restartTelemetryLoop();
}

// Safe bootstrap regardless of readyState
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    init();
  });
} else {
  init();
}
