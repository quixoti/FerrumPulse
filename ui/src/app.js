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
    setTab('tab-settings');
  } else if (!e.ctrlKey && !e.altKey && !e.metaKey && e.key >= '1' && e.key <= '7') {
    if (document.activeElement?.tagName === 'INPUT' || document.activeElement?.tagName === 'TEXTAREA') return;
    const tabOrder = ['tab-overview', 'tab-cpu', 'tab-process', 'tab-tweaks', 'tab-storage', 'tab-battery', 'tab-settings'];
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

    const autostartText = tw.autostart_enabled ? 'Autostart Active ✓' : 'Enable Autostart';
    if (el.btnToggleAutostart) {
      el.btnToggleAutostart.textContent = autostartText;
      el.btnToggleAutostart.classList.toggle('active', !!tw.autostart_enabled);
    }
    if (el.btnToggleAutostartOverview) {
      el.btnToggleAutostartOverview.textContent = autostartText;
      el.btnToggleAutostartOverview.classList.toggle('active', !!tw.autostart_enabled);
    }
  } catch (err) {
    console.error('Tweaks refresh error:', err);
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
        if (window.__TAURI__?.window?.getCurrentWindow) {
          window.__TAURI__.window.getCurrentWindow().destroy();
        } else {
          window.close();
        }
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
      await invoke('cmd_toggle_global_timer', { enable: false });
      await invoke('cmd_toggle_raw_mouse', { enable: false });
      showToast('All system presets restored to Windows defaults.');
    } catch (e) {
      showToast(`Revert error: ${e}`);
    }
    refreshTweaks();
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
        cleanD3dCache: el.chkCleanD3d ? el.chkCleanD3d.checked : true,
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

// Mock Fallback (used only when opened directly in a browser without Tauri)
function mockHandler(cmd, args) {
  switch (cmd) {
    case 'cmd_get_power_status':
      return { active_scheme_name: 'High Performance', active_scheme_guid: '8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c', min_cores_ac_percent: 100, is_unparked: true };
    case 'cmd_get_memory_stats':
      return { total_ram_mb: 32768, available_ram_mb: 22400, used_ram_mb: 10368, memory_load_percent: 32 };
    case 'cmd_get_daemon_status':
      return { is_running: true, active_game: null, boosted_pid: null, throttled_count: 0 };
    case 'cmd_get_daemon_config':
      return { enabled: true, boost_priority: true, throttle_background: true, target_games: ['cs2.exe', 'valorant.exe', 'cyberpunk2077.exe', 'apex.exe', 'dota2.exe'], throttled_apps: ['chrome.exe', 'discord.exe'] };
    case 'cmd_get_tweaks_status':
      return { diagtrack_disabled: false, gamedvr_disabled: false, network_pacing_optimized: false, global_timer_enabled: true, raw_mouse_enabled: true, autostart_enabled: false };
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
    case 'cmd_set_display_refresh_rate':
      return { success: true };
    case 'cmd_toggle_global_timer':
      return { success: true, message: args.enable ? 'Global timer enabled.' : 'Global timer reverted.' };
    case 'cmd_toggle_raw_mouse':
      return { success: true, message: args.enable ? 'Raw 1:1 mouse input enabled.' : 'Mouse acceleration restored.' };
    case 'cmd_toggle_autostart':
      return { success: true, message: args.enable ? 'Autostart enabled.' : 'Autostart disabled.' };
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

  // Settled initial fetch so a single delay never blocks the rest of the app
  await Promise.allSettled([
    refreshPower(),
    refreshMemory(),
    refreshDaemon(),
    refreshDaemonConfig(),
    refreshTweaks(),
    refreshStorage(),
    refreshBattery(),
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
