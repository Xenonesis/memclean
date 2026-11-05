/**
 * MemClean — Interactive 5-Tier NT Kernel Memory Simulator Component
 */

'use strict';

export function initMemorySimulator() {
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
