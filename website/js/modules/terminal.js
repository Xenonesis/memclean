/**
 * MemClean — Interactive CLI Terminal Showcase Component
 */

'use strict';

import { copyToClipboard } from './interactions.js';

export function initTerminalShowcase() {
  const tabs = document.querySelectorAll('.terminal-tab-btn');
  const cmdText = document.getElementById('current-cmd-text');
  const outputText = document.getElementById('terminal-output');
  const copyBtn = document.getElementById('copy-terminal-btn');

  if (!tabs.length || !cmdText || !outputText) return;

  const COMMAND_DATA = {
    status: {
      cmd: 'memclean.exe --status',
      output: `MemClean - Ultra-Lightweight Pure Rust Memory Optimizer v1.1.0
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
      output: `[INFO] Initializing MemClean v1.1.0...
[INFO] Acquired SeProfileSingleProcessPrivilege & SeIncreaseQuotaPrivilege.
[INFO] Purging Standby List (NtSetSystemInformation)... OK
[INFO] Purging Low-Priority Standby List (Priority 0)... OK
[INFO] Flushing Modified Page List to disk... OK
[INFO] Trimming System File Cache working set... OK
[INFO] Optimization complete!
Physical RAM Freed: 7,824 MB
Available Physical RAM: 10,640 MB / 16,000 MB (34% used)`
    },
    network: {
      cmd: 'memclean.exe --network-status',
      output: `=== MemClean Competitive Network Latency Status ===
Network Throttling Index: Some(0xFFFFFFFF) (Disabled: true)
System Responsiveness   : Some(0) (Gaming 100%: true)
Active Interfaces       : 2 detected (Optimized: 2/7)
Rollback Backup Exists  : true
  - [{2af6ca1] IP: 192.168.1.25 | TcpAckFrequency: 1 | TCPNoDelay: 1 | Optimized: true
  - [{303fd9f] IP: 192.168.1.65 | TcpAckFrequency: 1 | TCPNoDelay: 1 | Optimized: true`
    },
    vram: {
      cmd: 'memclean.exe --vram',
      output: `=== MemClean GPU & Video Memory (VRAM) Diagnostics ===
[0] Intel(R) Iris(R) Xe Graphics (Vendor: 0x8086, Device: 0x46A6)
    Dedicated VRAM       : 128 MB
    Dedicated System RAM : 0 MB
    Shared System RAM    : 8044 MB
[1] NVIDIA GeForce RTX 3060 Laptop GPU (Vendor: 0x10DE, Device: 0x2560)
    Dedicated VRAM       : 5994 MB
    Dedicated System RAM : 0 MB
    Shared System RAM    : 8044 MB`
    },
    cpu: {
      cmd: 'memclean.exe --cpu-topology',
      output: `=== MemClean CPU Topology & Hybrid Architecture ===
Physical Cores     : 14 (P-Cores: 6, E-Cores: 8)
Logical Processors : 20 threads
Architecture Type  : Intel/AMD Hybrid Core Architecture
All-Cores Mask     : 0xFFFFF
P-Cores Affinity   : 0xFFF
E-Cores Affinity   : 0xFF000
  Core [ 0.. 5]: P-Core | Class: 1 | SMT: true  | Mask: 0x0FFF
  Core [ 6..13]: E-Core | Class: 0 | SMT: false | Mask: 0xFF000`
    },
    telemetry: {
      cmd: 'memclean.exe --telemetry',
      output: `=== MemClean Live Memory Telemetry & Stutter Risk ===
Sampling Duration       : 1000 ms
Hard Page Faults Rate   : 2 / sec (Disk Reads)
Hard Fault I/O Ops Rate : 47384 / sec
Total Page Faults Rate  : 9711 / sec
Context Switches Rate   : 1857 / sec
Stutter Risk Level      : Smooth (Zero Disk Stutter)`
    },
    hud: {
      cmd: 'memclean.exe --hud',
      output: `[INFO] Initializing Zero-Injection Minimalist Gaming HUD Overlay...
[INFO] SetLayeredWindowAttributes(hwnd, 0, 225, LWA_ALPHA)... SUCCESS
[INFO] Win32 Click-Through enabled: WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOPMOST
[INFO] Global Hotkey registered: Ctrl + Shift + O (Toggle HUD)
Gaming HUD Active in screen corner. 100% Anti-Cheat Safe.`
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
