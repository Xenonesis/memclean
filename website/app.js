/**
 * MemClean — Interactive Website Engine
 * Zero-dependency pure vanilla JavaScript.
 *
 * Features:
 * 1. 5-Tier NT Kernel Memory Optimizer Simulation (with micro-delay animation)
 * 2. Real-time Telemetry & Dynamic GDI Tray Badge Preview
 * 3. Scriptable CLI Terminal Command Switcher with authentic stdout
 * 4. Universal One-Click Clipboard Copy with Toast Feedback
 * 5. Responsive Mobile Navigation Menu & Smooth Anchor Scrolling
 */

'use strict';

document.addEventListener('DOMContentLoaded', () => {
  initMemorySimulator();
  initTerminalShowcase();
  initClipboardHandlers();
  initMobileNav();
  initSmoothScroll();
  initTaskbarClock();
});

/* ==========================================================================
   1. Interactive 5-Tier Memory Optimizer Simulation
   ========================================================================== */

function initMemorySimulator() {
  const cleanBtn = document.getElementById('sim-clean-btn') || document.getElementById('btnRunOptimize');
  const resetBtn = document.getElementById('sim-reset-btn') || document.getElementById('btnResetDemo');
  const ramPercentEl = document.getElementById('sim-ram-percent') || document.getElementById('ramPercentage');
  const ramUsageEl = document.getElementById('sim-ram-usage') || document.getElementById('ramUsedGB');
  const progressTrack = document.getElementById('sim-progress-track');

  const segActive = document.getElementById('sim-seg-active');
  const segStandby = document.getElementById('sim-seg-standby');
  const segModified = document.getElementById('sim-seg-modified');
  const segFree = document.getElementById('sim-seg-free');

  const legendActive = document.getElementById('legend-active');
  const legendStandby = document.getElementById('legend-standby');
  const legendModified = document.getElementById('legend-modified');
  const legendFree = document.getElementById('legend-free');

  const freedBadge = document.getElementById('sim-freed-badge') || document.getElementById('freedAmount');
  const freedText = document.getElementById('freed-amount-text') || (freedBadge ? freedBadge.querySelector('span') : null);

  const logStatus = document.getElementById('sim-log-status');
  const logSteps = [
    document.getElementById('log-step-1'),
    document.getElementById('log-step-2'),
    document.getElementById('log-step-3'),
    document.getElementById('log-step-4'),
    document.getElementById('log-step-5')
  ];

  const trayIcon = document.getElementById('sim-tray-icon');
  const trayNum = document.getElementById('tray-num');

  if (!cleanBtn || !ramPercentEl) return;

  // Initial State Data
  const INITIAL_STATE = {
    percent: 82,
    usedGB: 13.2,
    totalGB: 16.0,
    activeWidth: 32,
    activeGB: 5.1,
    standbyWidth: 38,
    standbyGB: 6.1,
    modifiedWidth: 12,
    modifiedGB: 1.9,
    freeWidth: 18,
    freeGB: 2.9,
    trayNum: 82,
    isOptimal: false
  };

  // 5 Purge Stages
  const PURGE_STEPS = [
    {
      stepIndex: 0,
      runningText: 'Purging standby (prio 0-7)...',
      doneText: 'Done (-3,900 MB)',
      delay: 350,
      state: {
        percent: 58,
        usedGB: 9.3,
        activeWidth: 32,
        activeGB: 5.1,
        standbyWidth: 14,
        standbyGB: 2.2,
        modifiedWidth: 12,
        modifiedGB: 1.9,
        freeWidth: 42,
        freeGB: 6.8,
        trayNum: 58
      }
    },
    {
      stepIndex: 1,
      runningText: 'Purging zero-tier pages...',
      doneText: 'Done (-1,500 MB)',
      delay: 320,
      state: {
        percent: 49,
        usedGB: 7.8,
        activeWidth: 32,
        activeGB: 5.1,
        standbyWidth: 5,
        standbyGB: 0.8,
        modifiedWidth: 12,
        modifiedGB: 1.9,
        freeWidth: 51,
        freeGB: 8.2,
        trayNum: 49
      }
    },
    {
      stepIndex: 2,
      runningText: 'Writing dirty pages to disk...',
      doneText: 'Done (-1,600 MB)',
      delay: 360,
      state: {
        percent: 39,
        usedGB: 6.2,
        activeWidth: 32,
        activeGB: 5.1,
        standbyWidth: 5,
        standbyGB: 0.8,
        modifiedWidth: 2,
        modifiedGB: 0.3,
        freeWidth: 61,
        freeGB: 9.8,
        trayNum: 39
      }
    },
    {
      stepIndex: 3,
      runningText: 'Trimming kernel cache...',
      doneText: 'Done (-500 MB)',
      delay: 300,
      state: {
        percent: 36,
        usedGB: 5.7,
        activeWidth: 29,
        activeGB: 4.6,
        standbyWidth: 5,
        standbyGB: 0.8,
        modifiedWidth: 2,
        modifiedGB: 0.3,
        freeWidth: 64,
        freeGB: 10.3,
        trayNum: 36
      }
    },
    {
      stepIndex: 4,
      runningText: 'Shielding Cyberpunk2077.exe...',
      doneText: 'Shielded & Done (-300 MB)',
      delay: 350,
      state: {
        percent: 34,
        usedGB: 5.4,
        activeWidth: 27,
        activeGB: 4.3,
        standbyWidth: 5,
        standbyGB: 0.8,
        modifiedWidth: 2,
        modifiedGB: 0.3,
        freeWidth: 66,
        freeGB: 10.6,
        trayNum: 34,
        isOptimal: true
      }
    }
  ];

  let isRunning = false;
  let activeAnimFrame = null;
  let currentDisplayPercent = INITIAL_STATE.percent;
  let currentDisplayGB = INITIAL_STATE.usedGB;

  // Number animation helper using requestAnimationFrame
  function animateValue(from, to, duration, onUpdate, onComplete) {
    const startTime = performance.now();
    function tick(now) {
      const elapsed = now - startTime;
      const progress = Math.min(elapsed / duration, 1);
      // Ease-out cubic formula
      const ease = 1 - Math.pow(1 - progress, 3);
      const current = from + (to - from) * ease;
      onUpdate(current);
      if (progress < 1) {
        activeAnimFrame = requestAnimationFrame(tick);
      } else {
        if (onComplete) onComplete();
      }
    }
    activeAnimFrame = requestAnimationFrame(tick);
  }

  function applyMemoryState(state, animateNumbers = true) {
    // Update progress bar segments
    if (segActive) {
      segActive.style.width = `${state.activeWidth}%`;
      segActive.setAttribute('title', `Active Working Sets: ${state.activeGB.toFixed(1)} GB`);
    }
    if (segStandby) {
      segStandby.style.width = `${state.standbyWidth}%`;
      segStandby.setAttribute('title', `Standby List Cache: ${state.standbyGB.toFixed(1)} GB`);
    }
    if (segModified) {
      segModified.style.width = `${state.modifiedWidth}%`;
      segModified.setAttribute('title', `Modified Dirty List: ${state.modifiedGB.toFixed(1)} GB`);
    }
    if (segFree) {
      segFree.style.width = `${state.freeWidth}%`;
      segFree.setAttribute('title', `Free / Available RAM: ${state.freeGB.toFixed(1)} GB`);
    }

    // Update legends
    if (legendActive) legendActive.textContent = `Active Apps (${state.activeGB.toFixed(1)} GB)`;
    if (legendStandby) legendStandby.textContent = `Standby Cache (${state.standbyGB.toFixed(1)} GB)`;
    if (legendModified) legendModified.textContent = `Modified List (${state.modifiedGB.toFixed(1)} GB)`;
    if (legendFree) legendFree.textContent = `Free List (${state.freeGB.toFixed(1)} GB)`;

    if (progressTrack) {
      progressTrack.setAttribute('aria-valuenow', Math.round(state.percent));
    }

    // Update numbers
    if (animateNumbers) {
      const startPercent = currentDisplayPercent;
      const targetPercent = state.percent;
      const startGB = currentDisplayGB;
      const targetGB = state.usedGB;

      animateValue(startPercent, targetPercent, 350, (val) => {
        currentDisplayPercent = val;
        ramPercentEl.textContent = `${Math.round(val)}%`;
      });

      animateValue(startGB, targetGB, 350, (val) => {
        currentDisplayGB = val;
        if (ramUsageEl) {
          ramUsageEl.textContent = `${val.toFixed(1)} GB / 16.0 GB Used`;
        }
      });
    } else {
      currentDisplayPercent = state.percent;
      currentDisplayGB = state.usedGB;
      ramPercentEl.textContent = `${Math.round(state.percent)}%`;
      if (ramUsageEl) {
        ramUsageEl.textContent = `${state.usedGB.toFixed(1)} GB / 16.0 GB Used`;
      }
    }

    // Update Tray Preview
    if (trayNum) {
      trayNum.textContent = state.trayNum;
    }
    if (trayIcon) {
      if (state.isOptimal) {
        trayIcon.classList.add('optimal');
        trayIcon.setAttribute('title', `MemClean: ${state.trayNum}% RAM Used (Optimal)`);
      } else {
        trayIcon.classList.remove('optimal');
        trayIcon.setAttribute('title', `MemClean: ${state.trayNum}% RAM Used`);
      }
    }
  }

  function setLogStepState(index, status, customText = '') {
    const stepEl = logSteps[index];
    if (!stepEl) return;

    const statusEl = stepEl.querySelector('.log-step-status');
    stepEl.classList.remove('active', 'completed');

    if (status === 'active') {
      stepEl.classList.add('active');
      if (statusEl) {
        statusEl.textContent = customText || 'Running...';
        statusEl.className = 'log-step-status';
        statusEl.style.color = 'var(--color-accent)';
      }
    } else if (status === 'completed') {
      stepEl.classList.add('completed');
      if (statusEl) {
        statusEl.textContent = customText || 'Done';
        statusEl.className = 'log-step-status';
        statusEl.style.color = '#22C55E';
      }
    } else {
      // Idle
      if (statusEl) {
        statusEl.textContent = 'Idle';
        statusEl.className = 'log-step-status stdout-dim';
        statusEl.style.color = '';
      }
    }
  }

  function resetSimulation() {
    cancelAnimationFrame(activeAnimFrame);
    activeAnimFrame = null;
    isRunning = false;

    // Reset button states
    cleanBtn.disabled = false;
    cleanBtn.classList.remove('btn-loading');
    cleanBtn.innerHTML = `
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
      </svg>
      <span>Clean Memory Now</span>
    `;

    // Hide freed badge
    if (freedBadge) {
      freedBadge.classList.remove('show');
    }

    // Reset log steps
    if (logStatus) {
      logStatus.textContent = 'Waiting for trigger';
      logStatus.className = 'stdout-dim';
      logStatus.style.color = '';
    }

    for (let i = 0; i < logSteps.length; i++) {
      setLogStepState(i, 'idle');
    }

    // Restore initial values
    applyMemoryState(INITIAL_STATE, false);
  }

  async function runSimulation() {
    if (isRunning) return;
    isRunning = true;

    // Button in running state
    cleanBtn.disabled = true;
    cleanBtn.innerHTML = `
      <svg class="spin-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" style="animation: spin 1s linear infinite;">
        <circle cx="12" cy="12" r="10" stroke-opacity="0.25"></circle>
        <path d="M12 2a10 10 0 0 1 10 10"></path>
      </svg>
      <span>Purging Tiers...</span>
    `;

    if (logStatus) {
      logStatus.textContent = 'Purging memory tiers...';
      logStatus.className = '';
      logStatus.style.color = 'var(--color-accent)';
    }

    if (freedBadge) {
      freedBadge.classList.remove('show');
    }

    // Run sequential steps with micro delays
    for (let i = 0; i < PURGE_STEPS.length; i++) {
      const step = PURGE_STEPS[i];

      // Mark step active
      setLogStepState(i, 'active', step.runningText);

      // Wait micro delay
      await new Promise((resolve) => setTimeout(resolve, step.delay));

      // Update memory telemetry
      applyMemoryState(step.state, true);

      // Mark step completed
      setLogStepState(i, 'completed', step.doneText);

      // Brief pause between stages
      await new Promise((resolve) => setTimeout(resolve, 80));
    }

    // Final Completion State
    if (logStatus) {
      logStatus.textContent = '5-Tier Purge Complete';
      logStatus.style.color = '#22C55E';
    }

    // Animate freed badge
    if (freedBadge) {
      if (freedText) {
        animateValue(0, 7800, 600, (val) => {
          freedText.textContent = `+${Math.round(val).toLocaleString()} MB Reclaimed`;
        });
      }
      freedBadge.classList.add('show');
    }

    // Restore clean button for replay
    cleanBtn.disabled = false;
    cleanBtn.innerHTML = `
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
      </svg>
      <span>Clean Again</span>
    `;

    isRunning = false;
  }

  cleanBtn.addEventListener('click', runSimulation);

  if (resetBtn) {
    resetBtn.addEventListener('click', resetSimulation);
  }
}

/* ==========================================================================
   2. Scriptable CLI Terminal Command Switcher
   ========================================================================== */

function initTerminalShowcase() {
  const tabButtons = document.querySelectorAll('.terminal-tab-btn');
  const currentCmdText = document.getElementById('current-cmd-text');
  const terminalOutput = document.getElementById('terminal-output') || document.getElementById('terminalOutput');
  const copyTerminalBtn = document.getElementById('copy-terminal-btn') || document.getElementById('btnCopyTerminal');

  if (!tabButtons.length || !terminalOutput) return;

  const COMMAND_DATA = {
    status: {
      cmd: 'memclean.exe --status',
      output: `MemClean - Ultra-Lightweight Pure Rust Memory Optimizer v1.0.0
RAM Used:   13184 MB (82%)
RAM Avail:  2816 MB
RAM Total:  16000 MB

[Cache Breakdown]
Standby Cache: 6140 MB
Modified List: 1920 MB
Working Sets:  5124 MB`
    },
    clean: {
      cmd: 'memclean.exe --clean',
      output: `[+] Acquired SeDebugPrivilege, SeIncreaseQuotaPrivilege
[1/5] Purged Standby List (NtSetSystemInformation) -> 5,520 MB reclaimed
[2/5] Purged Low-Priority Standby (Priority 0)     -> 1,100 MB reclaimed
[3/5] Flushed Modified Page List to disk           -> 1,600 MB written
[4/5] Trimmed System File Cache (SetSystemFileCacheSize) -> 380 MB reclaimed
[5/5] Trimmed 34 background working sets (Shielded: Cyberpunk2077.exe)

Optimization completed: CleanResult { standby_purged: true, modified_flushed: true, file_cache_trimmed: true, processes_trimmed: 34 }
Reclaimed:  7,800 MB
RAM Used:   5,384 MB (34%)
RAM Avail:  10,616 MB`
    },
    aggressive: {
      cmd: 'memclean.exe --aggressive',
      output: `[!] WARNING: Aggressive mode overrides Game Guard restrictions.
[+] Purging all standby priority tiers (0-7)... OK
[+] Forcing synchronous modified page write-back... OK
[+] Purging system file cache & driver paged pools... OK
[+] Trimming ALL process working sets (including background daemons)... OK (108 processes trimmed)

Aggressive clean completed: CleanResult { standby_purged: true, modified_flushed: true, file_cache_trimmed: true, processes_trimmed: 108 }
Reclaimed:  8,420 MB
RAM Used:   4,764 MB (29%)
RAM Avail:  11,236 MB`
    },
    autostart: {
      cmd: 'memclean.exe --autostart',
      output: `[+] Creating elevated Task Scheduler job: "MemCleanAutoStart"
[+] Target: "C:\\Program Files\\MemClean\\memclean.exe" --minimized
[+] Trigger: At logon of any user
[+] RunLevel: HighestAvailable (Zero-UAC Silent Startup)
Registered elevated task scheduler autostart: ExitStatus(unix_wait_status(0))

SUCCESS: MemClean daemon will start silently in system tray at Windows logon.`
    }
  };

  let activeKey = 'status';

  function activateTab(key) {
    const data = COMMAND_DATA[key];
    if (!data) return;

    activeKey = key;

    tabButtons.forEach((btn) => {
      const match = (btn.dataset.command === key || btn.dataset.cmd === key);
      btn.classList.toggle('active', match);
      btn.setAttribute('aria-selected', match ? 'true' : 'false');
    });

    if (currentCmdText) {
      currentCmdText.textContent = data.cmd;
    }
    terminalOutput.textContent = data.output;
  }

  tabButtons.forEach((btn, index) => {
    btn.addEventListener('click', () => {
      const cmdKey = btn.dataset.command || btn.dataset.cmd || 'status';
      activateTab(cmdKey);
    });

    // Keyboard navigation (Arrow keys Left / Right, Home, End)
    btn.addEventListener('keydown', (e) => {
      let nextIndex = null;
      if (e.key === 'ArrowRight') {
        nextIndex = (index + 1) % tabButtons.length;
      } else if (e.key === 'ArrowLeft') {
        nextIndex = (index - 1 + tabButtons.length) % tabButtons.length;
      } else if (e.key === 'Home') {
        nextIndex = 0;
      } else if (e.key === 'End') {
        nextIndex = tabButtons.length - 1;
      }

      if (nextIndex !== null) {
        e.preventDefault();
        tabButtons[nextIndex].focus();
        const cmdKey = tabButtons[nextIndex].dataset.command || tabButtons[nextIndex].dataset.cmd;
        activateTab(cmdKey);
      }
    });
  });

  // Copy active terminal command
  if (copyTerminalBtn) {
    copyTerminalBtn.addEventListener('click', () => {
      const data = COMMAND_DATA[activeKey];
      const cmdToCopy = data ? data.cmd : (currentCmdText ? currentCmdText.textContent : 'memclean.exe');
      copyToClipboard(cmdToCopy, copyTerminalBtn);
    });
  }
}

/* ==========================================================================
   3. Universal One-Click Clipboard Copy with Toast Feedback
   ========================================================================== */

let toastTimeout = null;

function showToast(message = 'Copied to clipboard!') {
  const toast = document.getElementById('toast') || document.getElementById('copyFeedbackToast');
  const toastText = document.getElementById('toast-text') || (toast ? toast.querySelector('span') : null);

  if (!toast) return;

  if (toastText) {
    toastText.textContent = message;
  }

  toast.classList.add('show');
  toast.setAttribute('aria-hidden', 'false');

  clearTimeout(toastTimeout);

  toastTimeout = setTimeout(() => {
    toast.classList.remove('show');
    toast.setAttribute('aria-hidden', 'true');
  }, 2200);
}

function copyToClipboard(text, triggerBtn = null) {
  if (!text) return;

  function onSuccess() {
    showToast('Copied to clipboard!');
    if (triggerBtn) {
      triggerBtn.classList.add('copied');
      const originalSvg = triggerBtn.innerHTML;
      // Temporary checkmark icon
      triggerBtn.innerHTML = `
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="#22C55E" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
      `;
      setTimeout(() => {
        triggerBtn.innerHTML = originalSvg;
        triggerBtn.classList.remove('copied');
      }, 1500);
    }
  }

  if (navigator.clipboard && window.isSecureContext) {
    navigator.clipboard.writeText(text).then(onSuccess).catch(() => {
      fallbackCopyText(text, onSuccess);
    });
  } else {
    fallbackCopyText(text, onSuccess);
  }
}

function fallbackCopyText(text, callback) {
  try {
    const textArea = document.createElement('textarea');
    textArea.value = text;
    textArea.style.position = 'fixed';
    textArea.style.left = '-9999px';
    textArea.style.top = '-9999px';
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    const successful = document.execCommand('copy');
    document.body.removeChild(textArea);
    if (successful && callback) callback();
  } catch (err) {
    console.error('Fallback clipboard copy failed:', err);
  }
}

function initClipboardHandlers() {
  // Delegate click for snippet copy buttons and [data-copy]
  document.addEventListener('click', (e) => {
    const copyBtn = e.target.closest('[data-copy], .btn-copy-snippet');
    if (!copyBtn) return;

    const copyText = copyBtn.getAttribute('data-copy') ||
      (copyBtn.previousElementSibling && copyBtn.previousElementSibling.textContent.trim());

    if (copyText) {
      copyToClipboard(copyText, copyBtn);
    }
  });
}

/* ==========================================================================
   4. Responsive Mobile Navigation Menu
   ========================================================================== */

function initMobileNav() {
  const toggleBtn = document.getElementById('nav-toggle');
  const navLinks = document.getElementById('nav-links');

  if (!toggleBtn || !navLinks) return;

  function toggleMenu(forceState) {
    const isOpen = typeof forceState === 'boolean' ? forceState : !navLinks.classList.contains('active');
    navLinks.classList.toggle('active', isOpen);
    toggleBtn.setAttribute('aria-expanded', isOpen ? 'true' : 'false');
  }

  toggleBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    toggleMenu();
  });

  // Close when clicking any nav link
  navLinks.querySelectorAll('a').forEach((link) => {
    link.addEventListener('click', () => {
      toggleMenu(false);
    });
  });

  // Close when clicking outside
  document.addEventListener('click', (e) => {
    if (navLinks.classList.contains('active') && !navLinks.contains(e.target) && !toggleBtn.contains(e.target)) {
      toggleMenu(false);
    }
  });

  // Close on Escape key
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && navLinks.classList.contains('active')) {
      toggleMenu(false);
      toggleBtn.focus();
    }
  });
}

/* ==========================================================================
   5. Smooth Anchor Scrolling
   ========================================================================== */

function initSmoothScroll() {
  document.querySelectorAll('a[href^="#"]').forEach((anchor) => {
    anchor.addEventListener('click', function (e) {
      const href = this.getAttribute('href');
      if (href === '#' || href === '') return;

      const targetEl = document.querySelector(href);
      if (targetEl) {
        e.preventDefault();
        targetEl.scrollIntoView({
          behavior: 'smooth',
          block: 'start'
        });

        // Set focus for keyboard accessibility
        if (!targetEl.hasAttribute('tabindex')) {
          targetEl.setAttribute('tabindex', '-1');
        }
        targetEl.focus({ preventScroll: true });

        // Update URL hash without jumping
        if (history.pushState) {
          history.pushState(null, null, href);
        }
      }
    });
  });
}

/* ==========================================================================
   6. Live Taskbar Clock Sync
   ========================================================================== */

function initTaskbarClock() {
  const clockContainer = document.querySelector('.taskbar-clock');
  if (!clockContainer) return;

  function updateClock() {
    const now = new Date();
    let hours = now.getHours();
    const minutes = String(now.getMinutes()).padStart(2, '0');
    const ampm = hours >= 12 ? 'PM' : 'AM';
    hours = hours % 12 || 12;

    const month = String(now.getMonth() + 1).padStart(2, '0');
    const day = String(now.getDate()).padStart(2, '0');
    const year = now.getFullYear();

    const timeDiv = clockContainer.children[0] || document.createElement('div');
    const dateDiv = clockContainer.children[1] || document.createElement('div');

    timeDiv.textContent = `${hours}:${minutes} ${ampm}`;
    dateDiv.textContent = `${month}/${day}/${year}`;

    if (!clockContainer.children.length) {
      clockContainer.appendChild(timeDiv);
      clockContainer.appendChild(dateDiv);
    }
  }

  updateClock();
  setInterval(updateClock, 10000);
}
