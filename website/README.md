# MemClean Official Showcase Website

> High-performance, zero-dependency landing page and interactive simulator for **MemClean** — the ultra-lightweight, pure Rust Windows memory optimizer.

[![GitHub Pages Deployment](https://img.shields.io/badge/Deploy-GitHub%20Pages-238636?style=flat-square&logo=github)](https://xenonesis.github.io/memclean/)
[![HTML5](https://img.shields.io/badge/HTML5-Semantic-E34F26?style=flat-square&logo=html5&logoColor=white)](index.html)
[![CSS3](https://img.shields.io/badge/CSS3-Modern%20Tokens-1572B6?style=flat-square&logo=css3&logoColor=white)](styles.css)
[![JavaScript](https://img.shields.io/badge/JavaScript-Vanilla%20ES6+-F7DF1E?style=flat-square&logo=javascript&logoColor=black)](app.js)
[![Accessibility](https://img.shields.io/badge/A11y-WCAG%202.1%20AA-059669?style=flat-square)](styles.css)

Live Site: **[https://xenonesis.github.io/memclean/](https://xenonesis.github.io/memclean/)**

---

## Table of Contents

- [Overview](#overview)
- [Architecture and Tech Stack](#architecture-and-tech-stack)
- [Directory Structure](#directory-structure)
- [Interactive Features](#interactive-features)
- [Local Development and Preview](#local-development-and-preview)
- [GitHub Pages Deployment](#github-pages-deployment)
- [Accessibility and Responsive Standards](#accessibility-and-responsive-standards)
- [Verification and QA](#verification-and-qa)

---

## Overview

The `website/` directory hosts the official landing page and web demonstration for **MemClean**. Designed to mirror the project's engineering philosophy — **zero bloat, instant execution, native speed** — the site requires **zero build tools, zero bundlers, and zero external runtime npm packages**.

### Key Highlights
- **Instant Load Time:** Loads in `< 100ms` with zero JavaScript compilation steps.
- **OLED Dark Aesthetic:** Custom-tailored dark theme tokens (`#0B0F17` canvas, `#111827` cards) optimized for developer ergonomics.
- **Interactive NT Kernel Simulator:** A live browser-based simulation demonstrating MemClean's 5-stage memory purging engine.
- **Simulated Windows 11 Taskbar:** Visual representation of MemClean running in the Windows notification area with live RAM counters.
- **Interactive Terminal:** Switchable CLI preview showcasing real command output with instant copy functionality.

---

## Architecture and Tech Stack

| Layer | Implementation | Details |
| :--- | :--- | :--- |
| **Markup** | Semantic HTML5 | Accessible landmark regions (`<header>`, `<main>`, `<section>`, `<footer>`), ARIA live regions, microdata. |
| **Styling** | Modern CSS3 | CSS Custom Properties (Tokens), CSS Grid, Flexbox, custom scrollbars, `@media (prefers-reduced-motion)`. |
| **Scripting**| Vanilla JavaScript (ES6+) | Native DOM APIs, `requestAnimationFrame` cubic easing, Clipboard API, zero runtime dependencies. |
| **Typography**| Google Fonts | `Inter` (UI) and `JetBrains Mono` (Terminal / Code) via preconnected asynchronous fonts. |
| **Icons** | Inline SVG | Scalable vector graphics embedded directly in HTML to avoid HTTP roundtrips and render flashes. |
| **CI/CD** | GitHub Actions | Automated build-free deployment to GitHub Pages via official GitHub Actions. |

---

## Directory Structure

```text
website/
├── index.html        # Main landing page markup (Hero, Features, Simulator, Architecture, Comparison, CLI, Download)
├── styles.css        # Complete OLED dark mode design system, responsive layout, animations, and tokens
├── app.js            # Interactive simulator engine, terminal tab manager, clipboard utility, and navigation
└── README.md         # Architecture documentation and local development guide (this file)
```

### Component Breakdown

1. **`index.html`**
   - **Hero Section:** Value proposition, release badges, primary download CTAs, and system requirements.
   - **Features Grid:** Key differentiators (Pure Rust, <500 KB binary, Game Guard, Zero-UAC Task Scheduler).
   - **Memory Purge Simulator:** Interactive memory bar with 4-segment memory distribution (Active, Standby, Modified, Free) and a realistic Windows 11 taskbar tray icon with a dynamic clock.
   - **5-Tier Architecture:** Step-by-step breakdown of undocumented NT kernel API routines (`NtSetSystemInformation`, `SetSystemFileCacheSize`, `EmptyWorkingSet`).
   - **Side-by-Side Comparison Matrix:** Detailed feature and performance metrics vs. legacy tools (Sordum ReduceMemory).
   - **CLI Reference and Interactive Terminal:** Tabbed command inspector with live stdout output and copy-to-clipboard buttons.
   - **Download Center:** Direct links to Windows Installer (`MemClean_Setup.exe`), Portable executable (`memclean.exe`), and GitHub releases.
   - **Footer:** Deep navigation links, license, and repository links.

2. **`styles.css`**
   - **Tokens:** Centralized CSS variables for palette (`--color-background`, `--color-card`, `--color-accent`, etc.), spacing, elevation, and typography.
   - **Glassmorphism & Gradients:** Subtle backdrop filters and emerald accent highlights.
   - **Responsive Breakpoints:**
     - Desktop: `> 1024px` (Full grid layouts, dual-pane simulator, complete navigation)
     - Tablet: `768px – 1024px` (Stacked simulator, responsive comparison table)
     - Mobile: `< 768px` (Collapsible hamburger drawer, stacked action buttons, touch-friendly tap targets)
     - Small Mobile: `< 480px` (Optimized font scale, full-width buttons, condensed terminal)

3. **`app.js`**
   - **Memory Simulator Engine (`initMemorySimulator`):** Simulates 5 distinct memory purge stages with cubic-eased numerical animation and sequential step logging.
   - **Terminal Showcase (`initTerminalShowcase`):** Manages CLI command switching with keyboard arrow navigation and active state synchronization.
   - **Clipboard Manager (`copyToClipboard`, `fallbackCopyText`):** Asynchronous clipboard write with visual feedback (button state toggle and floating toast notification).
   - **Mobile Drawer (`initMobileNav`):** Accessible toggle for mobile navigation with backdrop dismiss and Escape key listener.
   - **Taskbar Clock (`initTaskbarClock`):** Real-time clock emulation inside the simulated Windows taskbar.
   - **Smooth Scrolling (`initSmoothScroll`):** Intercepts anchor links for smooth scrolling with header offset compensation.

---

## Interactive Features

### 1. Live Memory Purge Simulator
- **Trigger:** Click **"Run Memory Purge"** in the simulator section.
- **Process:** The engine sequentially steps through 5 purge phases:
  1. `SYSTEM_MEMORY_LIST_PURGE` &bull; Priority 0-7 Standby Purge
  2. `SYSTEM_MEMORY_LIST_PURGE` &bull; Priority 0 Standby Purge (Cold cache)
  3. `SYSTEM_MEMORY_LIST_COMMAND` &bull; Flush Modified Page List
  4. `SetSystemFileCacheSize` &bull; Trim System File Cache
  5. `EmptyWorkingSet` &bull; Trim Background Working Sets
- **Visuals:** The 4-segment memory bar animates from 82% (13.2 GB) down to 24% (3.8 GB), updating active/standby/modified/free counters and the simulated Windows tray icon.

### 2. Tabbed CLI Terminal
- Allows testing and previewing CLI command outputs without running a local Windows shell:
  - `memclean.exe --status`
  - `memclean.exe --clean`
  - `memclean.exe --deep-clean`
  - `memclean.exe --help`
- Includes one-click copy with instant toast confirmation.

---

## Local Development and Preview

Because the site uses purely native web technologies, no build or compilation step is needed.

### Option 1: Direct File Opening
Double-click `website/index.html` or open it in any browser:
```bash
# Windows PowerShell
Start-Process "website/index.html"
```

### Option 2: Node.js / NPX
Serve the directory with an HTTP server to test live reload:
```bash
# Using 'serve'
npx serve website

# Or using 'http-server'
npx http-server website -p 3000 -o
```

### Option 3: Python Built-in Server
Available on almost every developer environment without installing dependencies:
```bash
# Python 3
python -m http.server 8000 --directory website
```
Then navigate to: `http://localhost:8000`

### Option 4: Bun
```bash
bun x serve website
```

### Option 5: VS Code Live Server
Right-click `website/index.html` in VS Code and select **"Open with Live Server"**.

---

## GitHub Pages Deployment

The repository includes a dedicated, automated GitHub Actions workflow at [`.github/workflows/deploy-site.yml`](../.github/workflows/deploy-site.yml).

### Workflow Details
- **Trigger:** Triggers on any `push` to the `master` or `main` branch that modifies files in `website/**` or the workflow file itself, or via manual **Run workflow** (`workflow_dispatch`).
- **Permissions:**
  - `contents: read`
  - `pages: write`
  - `id-token: write`
- **Actions Used:**
  - `actions/checkout@v4` — Clones repository files.
  - `actions/configure-pages@v5` — Configures GitHub Pages environment.
  - `actions/upload-pages-artifact@v3` — Packages the `website/` directory.
  - `actions/deploy-pages@v4` — Publishes artifact to GitHub Pages CDN.

### Setting Up GitHub Pages in the Repository
To enable Pages for your repository fork or origin:
1. Open the repository on GitHub.
2. Go to **Settings** > **Pages**.
3. Under **Build and deployment**:
   - **Source**: Select **GitHub Actions** (do *not* select "Deploy from a branch").
4. Push a commit to `master` (or trigger the workflow manually from the **Actions** tab).
5. The site will be published to `https://<username>.github.io/<repository>/`.

---

## Accessibility and Responsive Standards

- **Color Contrast:** All body text meets or exceeds the **4.5:1** contrast ratio required by WCAG 2.1 AA (tested on `#0B0F17` background with `#F8FAFC` foreground).
- **Reduced Motion:** Full support for `prefers-reduced-motion: reduce`. When enabled, animations and transitions are reduced to `0.01ms`, and numeric transitions snap immediately.
- **Keyboard Navigation:** All interactive elements (`<button>`, `<a>`, `.terminal-tab-btn`) are fully keyboard focusable with a distinct focus ring (`:focus-visible`).
- **Touch Targets:** All primary buttons and links provide a minimum touch target height of **46px** (`--btn` min-height: 46px).
- **ARIA Attributes:**
  - `aria-expanded` and `aria-controls` on mobile navigation toggle.
  - `aria-selected` and `role="tab"` on CLI terminal switcher.
  - `aria-live="polite"` on the simulator status log and freed memory badge.

---

## Verification and QA

Run this quick verification check to ensure all references remain intact:

```bash
# 1. Verify app.js syntax
node -c website/app.js

# 2. Check for missing local file references
# (All images and icons are embedded inline SVG; stylesheet is styles.css; script is app.js)
```

No external bundlers, polyfills, or node modules are required for production maintenance.
