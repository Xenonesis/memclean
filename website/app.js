/**
 * MemClean — Utilitarian Website Engine
 * Zero-dependency pure vanilla JavaScript.
 *
 * Features:
 * 1. 5-Tier NT Kernel Memory Optimizer Simulation
 * 2. Real-time Telemetry & Dynamic GDI Tray Badge Preview
 * 3. Scriptable CLI Terminal Command Switcher with authentic stdout
 * 4. Universal One-Click Clipboard Copy with Toast Feedback
 * 5. Utilitarian Accordion FAQ (with sharp + / − indicators)
 * 6. IntersectionObserver Scroll Entry Micro-Reveals
 * 7. Responsive Mobile Navigation Menu & Smooth Anchor Scrolling
 */

'use strict';

document.addEventListener('DOMContentLoaded', () => {
  initMemorySimulator();
  initTerminalShowcase();
  initClipboardHandlers();
  initFaqAccordion();
  initScrollReveals();
  initMobileNav();
  initSmoothScroll();
  initTaskbarClock();
});

/* ==========================================================================
   1. Interactive 5-Tier Memory Optimizer Simulation
   ========================================================================== */

function initMemorySimulator() {
  const cleanBtn = document.getElementById('sim-clean-btn');
  const resetBtn = document.getElementById('sim-reset-btn');
  const ramPercentEl = document.getElementById('sim-ram-percent');
  const ramUsageEl = document.getElementById('sim-ram-usage');
  const progressTrack = document.getElementById('sim-progress-track');

  const segActive = document.getElementById('sim-seg-active');
  const segStandby = document.getElementById('sim-seg-standby');
  const segModified = document.getElementById('sim-seg-modified');
  const segFree = document.getElementById('sim-seg-free');

  const legendActive = document.getElementById('legend-active');
  const legendStandby = document.getElementById('legend-standby');
  const legendModified = document.getElementById('legend-modified');
  const legendFree = document.getElementById('legend-free');

  const freedBadge = document.getElementById('sim-freed-badge');
  const freedText = document.getElementById('freed-amount-text');

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
      runningText: 'Flushing modified dirty pages...',
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
      runningText: 'Trimming filesystem cache...',
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
      segStandby.setAttribute('title', `Standby Page Cache: ${state.standbyGB.toFixed(1)} GB`);
    }
    if (segModified) {
      segModified.style.width = `${state.modifiedWidth}%`;
      segModified.setAttribute('title', `Modified Dirty List: ${state.modifiedGB.toFixed(1)} GB`);
    }
    if (segFree) {
      segFree.style.width = `${state.freeWidth}%`;
      segFree.setAttribute('title', `Free Available RAM: ${state.freeGB.toFixed(1)} GB`);
    }

    // Update legends
    if (legendActive) legendActive.textContent = `Active Working Sets (${state.activeGB.toFixed(1)} GB)`;
    if (legendStandby) legendStandby.textContent = `Standby Page Cache (${state.standbyGB.toFixed(1)} GB)`;
    if (legendModified) legendModified.textContent = `Modified Dirty List (${state.modifiedGB.toFixed(1)} GB)`;
    if (legendFree) legendFree.textContent = `Free Available RAM (${state.freeGB.toFixed(1)} GB)`;

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
      if (state.percent < 70) {
        trayIcon.style.backgroundColor = '#1F6C38';
        trayIcon.setAttribute('title', `MemClean: ${state.trayNum}% RAM Used (Healthy)`);
      } else if (state.percent < 85) {
        trayIcon.style.backgroundColor = '#855800';
        trayIcon.setAttribute('title', `MemClean: ${state.trayNum}% RAM Used (Elevated)`);
      } else {
        trayIcon.style.backgroundColor = '#9F2F2D';
        trayIcon.setAttribute('title', `MemClean: ${state.trayNum}% RAM Used (High)`);
      }
    }
  }

  function setLogStepState(index, status, customText = '') {
    const stepEl = logSteps[index];
    if (!stepEl) return;

    const statusEl = stepEl.querySelector('.log-step-status');
    stepEl.classList.remove('active', 'done');

    if (status === 'active') {
      stepEl.classList.add('active');
      if (statusEl) {
        statusEl.textContent = customText || 'Running...';
        statusEl.style.color = 'var(--pastel-blue-text)';
      }
    } else if (status === 'done') {
      stepEl.classList.add('done');
      if (statusEl) {
        statusEl.textContent = customText || 'Done';
        statusEl.style.color = 'var(--pastel-green-text)';
      }
    } else {
      if (statusEl) {
        statusEl.textContent = 'Idle';
        statusEl.style.color = '';
      }
    }
  }

  function resetToInitial() {
    if (isRunning) return;
    if (activeAnimFrame) cancelAnimationFrame(activeAnimFrame);

    applyMemoryState(INITIAL_STATE, true);

    if (logStatus) {
      logStatus.textContent = 'Waiting for trigger';
      logStatus.className = 'stdout-dim';
    }

    logSteps.forEach((_, idx) => setLogStepState(idx, 'idle'));

    if (freedBadge) {
      freedBadge.classList.remove('active');
    }

    cleanBtn.disabled = false;
    cleanBtn.style.opacity = '1';
  }

  function executeCleanSequence() {
    if (isRunning) return;
    isRunning = true;
    cleanBtn.disabled = true;
    cleanBtn.style.opacity = '0.6';

    if (freedBadge) freedBadge.classList.remove('active');
    if (logStatus) {
      logStatus.textContent = 'Pipeline Active';
      logStatus.className = 'stdout-dim';
    }

    let currentStep = 0;

    function runNextStep() {
      if (currentStep >= PURGE_STEPS.length) {
        // Complete
        isRunning = false;
        cleanBtn.disabled = false;
        cleanBtn.style.opacity = '1';
        if (logStatus) {
          logStatus.textContent = 'Completed Successfully';
          logStatus.style.color = 'var(--pastel-green-text)';
        }
        if (freedBadge) {
          if (freedText) freedText.textContent = '+7,800 MB Reclaimed';
          freedBadge.classList.add('active');
        }
        return;
      }

      const step = PURGE_STEPS[currentStep];
      setLogStepState(step.stepIndex, 'active', step.runningText);

      setTimeout(() => {
        applyMemoryState(step.state, true);
        setLogStepState(step.stepIndex, 'done', step.doneText);
        currentStep++;
        runNextStep();
      }, step.delay);
    }

    runNextStep();
  }

  cleanBtn.addEventListener('click', executeCleanSequence);
  if (resetBtn) {
    resetBtn.addEventListener('click', resetToInitial);
  }

  // Initial apply
  applyMemoryState(INITIAL_STATE, false);
}

/* ==========================================================================
   2. Scriptable CLI Terminal Command Switcher
   ========================================================================== */

function initTerminalShowcase() {
  const tabs = document.querySelectorAll('.terminal-tab-btn');
  const cmdText = document.getElementById('current-cmd-text');
  const outputText = document.getElementById('terminal-output');
  const copyBtn = document.getElementById('copy-terminal-btn');

  if (!tabs.length || !cmdText || !outputText) return;

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
      output: `[INFO] Initializing MemClean v1.0.0...
[INFO] Acquired SeProfileSingleProcessPrivilege & SeIncreaseQuotaPrivilege.
[INFO] Purging Standby List (NtSetSystemInformation)... OK
[INFO] Purging Low-Priority Standby List (Priority 0)... OK
[INFO] Flushing Modified Page List to disk... OK
[INFO] Trimming System File Cache working set... OK
[INFO] Optimization complete!
Physical RAM Freed: 7,824 MB
Available Physical RAM: 10,640 MB / 16,000 MB (34% used)`
    },
    aggressive: {
      cmd: 'memclean.exe --aggressive',
      output: `[WARN] Aggressive mode requested.
[INFO] Foreground Guard: DETECTED (Cyberpunk2077.exe [PID 14208]).
[INFO] Foreground working set protected from eviction.
[INFO] Purged 5-tier kernel cache.
[INFO] Trimmed 18 background processes.
Total Memory Reclaimed: 8,410 MB
Zero frame stutters detected.`
    },
    autostart: {
      cmd: 'memclean.exe --autostart',
      output: `[INFO] Registering Windows Task Scheduler logon task...
Task Name:    MemCleanLogonAutoStart
Run Level:    TASK_RUNLEVEL_HIGHEST
Logon Action: Launch 'C:\\Program Files\\MemClean\\memclean.exe'
Status:       Successfully configured. Zero UAC prompts will occur on reboot.`
    },
    json: {
      cmd: 'memclean.exe --stats --json',
      output: `{
  "timestamp": 1730310240,
  "memory": {
    "total_mb": 16088,
    "used_mb": 9236,
    "free_mb": 52,
    "zeroed_mb": 2004,
    "standby_mb": 4795,
    "modified_mb": 100,
    "used_percent": 57
  },
  "timer_resolution": {
    "current_ms": 0.500,
    "minimum_ms": 15.625,
    "maximum_ms": 0.500
  },
  "top_consumers": [
    { "pid": 10564, "name": "explorer.exe", "working_set_mb": 592 },
    { "pid": 7532, "name": "Code.exe", "working_set_mb": 362 }
  ]
}`
    },
    timer: {
      cmd: 'memclean.exe --timer-res 0.5',
      output: `[INFO] Initializing Windows high-resolution scheduler timer engine...
[INFO] Invoked NtSetTimerResolution(5000, 1)... SUCCESS
Active timer resolution: 0.500 ms (locked at 2,000 Hz)`
    },
    combine: {
      cmd: 'memclean.exe --combine',
      output: `[INFO] Invoked NtSetSystemInformation(130, SystemCombinePhysicalMemoryInformation)...
[INFO] Scanned physical RAM pages for identical read-only content.
Page combining executed: 14,210 duplicate pages merged (55.51 MB reclaimed).`
    }
  };

  tabs.forEach((tab) => {
    tab.addEventListener('click', () => {
      tabs.forEach((t) => {
        t.classList.remove('active');
        t.setAttribute('aria-selected', 'false');
      });

      tab.classList.add('active');
      tab.setAttribute('aria-selected', 'true');

      const key = tab.getAttribute('data-command');
      const data = COMMAND_DATA[key];
      if (data) {
        cmdText.textContent = data.cmd;
        outputText.textContent = data.output;
      }
    });
  });

  if (copyBtn) {
    copyBtn.addEventListener('click', () => {
      copyToClipboard(cmdText.textContent, copyBtn);
    });
  }
}

/* ==========================================================================
   3. Universal Clipboard Copy Handlers & Toast
   ========================================================================== */

let toastTimeout = null;

function showToast(message = 'Copied to clipboard') {
  const toast = document.getElementById('toast');
  const toastText = document.getElementById('toast-text');

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
  }, 2000);
}

function copyToClipboard(text, triggerBtn = null) {
  if (!text) return;

  function onSuccess() {
    showToast('Copied to clipboard');
    if (triggerBtn) {
      triggerBtn.classList.add('copied');
      const originalSvg = triggerBtn.innerHTML;
      triggerBtn.innerHTML = `
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
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
   4. Utilitarian Accordion FAQ
   ========================================================================== */

function initFaqAccordion() {
  const triggers = document.querySelectorAll('.faq-trigger');
  triggers.forEach((trigger) => {
    trigger.addEventListener('click', () => {
      const item = trigger.closest('.faq-item');
      if (!item) return;
      const isOpen = item.classList.contains('open');

      // Close all other items
      document.querySelectorAll('.faq-item.open').forEach((openItem) => {
        if (openItem !== item) {
          openItem.classList.remove('open');
          const t = openItem.querySelector('.faq-trigger');
          if (t) t.setAttribute('aria-expanded', 'false');
          const icon = openItem.querySelector('.faq-icon');
          if (icon) icon.textContent = '+';
        }
      });

      if (isOpen) {
        item.classList.remove('open');
        trigger.setAttribute('aria-expanded', 'false');
        const icon = trigger.querySelector('.faq-icon');
        if (icon) icon.textContent = '+';
      } else {
        item.classList.add('open');
        trigger.setAttribute('aria-expanded', 'true');
        const icon = trigger.querySelector('.faq-icon');
        if (icon) icon.textContent = '−';
      }
    });
  });
}

/* ==========================================================================
   5. IntersectionObserver Scroll Entry Micro-Reveals
   ========================================================================== */

function initScrollReveals() {
  const elements = document.querySelectorAll('.reveal-on-scroll');
  if (!('IntersectionObserver' in window)) {
    elements.forEach((el) => el.classList.add('revealed'));
    return;
  }

  const observer = new IntersectionObserver((entries, obs) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        entry.target.classList.add('revealed');
        obs.unobserve(entry.target);
      }
    });
  }, {
    threshold: 0.1,
    rootMargin: '0px 0px -40px 0px'
  });

  elements.forEach((el) => observer.observe(el));
}

/* ==========================================================================
   6. Responsive Mobile Navigation Menu
   ========================================================================== */

function initMobileNav() {
  const toggleBtn = document.getElementById('nav-toggle');
  const navLinks = document.getElementById('nav-links');

  if (!toggleBtn || !navLinks) return;

  function toggleMenu(forceState = null) {
    const isExpanded = forceState !== null ? forceState : toggleBtn.getAttribute('aria-expanded') !== 'true';
    toggleBtn.setAttribute('aria-expanded', isExpanded);
    navLinks.style.display = isExpanded ? 'flex' : '';
    if (isExpanded) {
      navLinks.style.flexDirection = 'column';
      navLinks.style.position = 'absolute';
      navLinks.style.top = '64px';
      navLinks.style.left = '0';
      navLinks.style.right = '0';
      navLinks.style.backgroundColor = 'var(--bg-surface)';
      navLinks.style.borderBottom = '1px solid var(--border-light)';
      navLinks.style.padding = '24px';
      navLinks.style.gap = '16px';
    }
  }

  toggleBtn.addEventListener('click', () => {
    toggleMenu();
  });

  navLinks.querySelectorAll('a').forEach((link) => {
    link.addEventListener('click', () => {
      toggleMenu(false);
    });
  });

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && toggleBtn.getAttribute('aria-expanded') === 'true') {
      toggleMenu(false);
      toggleBtn.focus();
    }
  });
}

/* ==========================================================================
   7. Smooth Anchor Scrolling
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

        if (!targetEl.hasAttribute('tabindex')) {
          targetEl.setAttribute('tabindex', '-1');
        }
        targetEl.focus({ preventScroll: true });

        if (history.pushState) {
          history.pushState(null, null, href);
        }
      }
    });
  });
}

/* ==========================================================================
   8. Live Taskbar Clock Sync
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
