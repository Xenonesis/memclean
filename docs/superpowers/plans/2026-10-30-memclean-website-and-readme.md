# MemClean Website & README Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a high-converting, developer-grade showcase website and an authoritative GitHub `README.md` for MemClean, following the OLED Dark Mode design system (`design-system/memclean/MASTER.md`) and UI/UX Pro Max guidelines.

**Architecture:** 
- Standalone zero-dependency static web application (`website/index.html`, `website/styles.css`, `website/app.js`) designed for instant local preview and direct deployment to GitHub Pages.
- OLED Dark Mode theme (`#0F172A` background, `#1E293B` cards, `#22C55E` accent green, `#F8FAFC` text) with pure SVG icons (Lucide/Heroicons), interactive real-time memory optimizer simulation widget, 5-tier kernel purge visualizer, interactive benchmark comparison table vs ReduceMemory, and one-click copyable CLI commands.
- High-impact GitHub repository `README.md` complete with SVG architecture diagrams, shields.io badges, benchmark comparison matrix, CLI reference, and zero-UAC scheduled task explanation.

**Tech Stack:** Semantic HTML5, Modern CSS3 with Custom Properties (OLED Dark Mode design tokens), Vanilla ECMAScript (zero build tooling required), SVG icon system, GitHub Actions (`.github/workflows/deploy-site.yml`).

**Spec:** `docs/superpowers/specs/2026-10-29-memclean-design.md` & `design-system/memclean/MASTER.md`

## Global Constraints

- **Design System Fidelity:** Strictly follow `design-system/memclean/MASTER.md` color tokens (`#0F172A`, `#1E293B`, `#22C55E`, `#F8FAFC`, `#EF4444`), typography (`Inter` + `JetBrains Mono`), and component shadows.
- **Icon Discipline:** NEVER use emojis as UI icons; use clean, accessible SVG vectors (Lucide/Heroicons standard).
- **Accessibility:** 4.5:1 minimum text contrast, visible `:focus-visible` keyboard focus rings, `prefers-reduced-motion` media queries, semantic ARIA attributes.
- **Performance & Zero Dependencies:** No heavy JS frameworks (React/Vue/Webpack/Vite) for the landing page; pure static HTML/CSS/JS that loads in < 50ms and works offline directly from the filesystem (`file://`) or GitHub Pages.
- **Cross-Device Responsiveness:** Pixel-perfect at 375px (mobile), 768px (tablet), 1024px (laptop), and 1440px (desktop).

---

### Task 1: Authoritative GitHub README.md

**Files:**
- Create: `README.md`
- Test: Throwaway markdown preview / linter check

**Interfaces:**
- Consumes: Features, architecture, CLI flags from `memclean/src/main.rs`, `memclean/src/core/ntdll.rs`, `memclean/src/engine/guard.rs`, and Inno Setup installer facts.
- Produces: GitHub repo front page with clear value proposition, quick start, installation options, CLI documentation, and benchmark comparison table.

- [ ] **Step 1: Write draft README structure**
  Include:
  1. Header with Shields.io badges (Rust 2021, Windows 10/11, License MIT, Binary Size < 500 KB, Zero UAC).
  2. One-sentence pitch: "An ultra-lightweight, native Rust Windows memory optimizer that replaces Sordum ReduceMemory with multi-tier NT kernel memory purging, zero-latency game guard, and zero-UAC boot startup."
  3. Why MemClean vs Sordum ReduceMemory (comparison table: Memory overhead, Engine type, Standby list purge, Low-priority standby, Modified list flush, File cache trim, Fullscreen game shield, Idle CPU, Binary size).
  4. 5-Tier Kernel Purge Architecture diagram (ASCII / Mermaid).
  5. Installation options:
     - Installer: `MemClean_Setup.exe` (with silent background startup task)
     - Portable standalone: `memclean.exe`
     - Build from source: `cargo build --release`
  6. CLI Commands Reference (`--status`, `--clean`, `--aggressive`, `--standby`, `--autostart`, `--no-autostart`).
  7. Configuration & Customization (`%APPDATA%\MemClean\config.toml`).
  8. Smart Game Guard explanation (fullscreen auto-detection + system critical process whitelist).
  9. License & Contributing.

- [ ] **Step 2: Verify all paths, links, and commands in README.md**
  Ensure all repository URLs point to `https://github.com/Xenonesis/memclean` and download links point to the latest release assets.

- [ ] **Step 3: Commit**
  ```bash
  git add README.md
  git commit -m "docs: add comprehensive developer README with benchmarks and architecture guide"
  ```

---

### Task 2: Showcase Landing Page HTML & Modern CSS Design Tokens

**Files:**
- Create: `website/index.html`
- Create: `website/styles.css`
- Assets: Inline SVG icons (Lucide standard: CPU, Shield, Zap, Terminal, Download, Github, CheckCircle, Sliders, Activity)

**Interfaces:**
- Consumes: `design-system/memclean/MASTER.md` CSS variables and layout structure.
- Produces: Semantic HTML5 document and CSS styling covering Hero, Feature Bento Grid, 5-Tier Architecture, Interactive Memory Simulator, Benchmark Matrix, CLI Terminal Showcase, and Download CTA.

- [ ] **Step 1: Create `website/styles.css` with OLED Dark Mode Design Tokens**
  - Define root variables (`--color-background: #0F172A`, `--color-card: #1B2336`, `--color-primary: #1E293B`, `--color-accent: #22C55E`, `--color-foreground: #F8FAFC`, `--color-border: #334155`).
  - Google Fonts import (`Inter:wght@400;500;600;700` and `JetBrains+Mono:wght@400;500`).
  - Reset, fluid typography scale, and responsive grid layouts.
  - Custom glassmorphic borders (`border: 1px solid rgba(255, 255, 255, 0.08)`).
  - Glowing accent CTA styles with smooth transitions (200ms ease) and visible keyboard `:focus-visible` focus rings.
  - Interactive comparison table styling with subtle hover highlights.
  - Responsive media queries (1024px, 768px, 480px, 375px).

- [ ] **Step 2: Create `website/index.html` Markup**
  - **Navigation Bar:** Logo (Memory Chip SVG), navigation links (Features, Architecture, Benchmarks, CLI, Docs), GitHub star button, and primary "Download Installer" CTA.
  - **Hero Section:**
    - High-impact headline: *"Reclaim Windows Memory. Without The Stutter."*
    - Subtitle: *"The ultra-lightweight (<500 KB), pure Rust memory optimizer that replaces legacy AutoIt tools with 5-tier kernel purging and smart fullscreen game protection."*
    - Action buttons: "Download v1.0.0 Setup (.exe)" (Primary Green) and "Portable Edition (.exe)" / "GitHub" (Secondary Slate).
    - Quick spec pill badges: `Rust 2021` • `< 2 MB RAM at Idle` • `0.0% CPU` • `Zero-UAC Boot Startup`.
  - **Interactive Hero Visualizer / Live Simulator:**
    - Interactive RAM meter mockup demonstrating the exact GDI tray badge and Win32 dashboard (before clean vs after clean).
  - **Feature Bento Grid:**
    1. *5-Tier NT Kernel Purge* (Standby list, Low-priority standby, Modified page flush, System file cache, Selective working set).
    2. *Smart Game Guard & Foreground Shield* (Auto-detects fullscreen games, prevents frame drops).
    3. *Dynamic GDI Tray Badge* (Live color-coded percentage: Green <70%, Yellow 70-85%, Red >85%).
    4. *Silent Zero-UAC Elevation* (Automated Windows Task Scheduler integration).
    5. *Dual-Mode Single Binary* (Runs as tray GUI or headless CLI via `--status` and `--clean`).
    6. *100% Native Rust & Win32* (No webview, no electron, zero runtime DLL dependencies).
  - **Architecture Deep-Dive Section:**
    - Visual flow of Windows NT memory lists: Standby List vs Modified List vs Working Sets.
  - **Benchmark Comparison Matrix:**
    - MemClean vs Sordum ReduceMemory vs Standard Working Set Cleaners.
  - **CLI Terminal Showcase:**
    - Interactive tabbed terminal showing real output for `memclean --status`, `memclean --clean`, and `memclean --aggressive` with copy button.
  - **Download & Getting Started Section:**
    - Direct installer download, portable binary download, and winget/cargo build snippets.
  - **Footer:** Links to GitHub, MIT License, Issue Tracker, and documentation.

- [ ] **Step 3: Validate HTML/CSS structure**
  Open and preview in browser; verify all SVG icons render cleanly without emoji fallbacks.

- [ ] **Step 4: Commit**
  ```bash
  git add website/index.html website/styles.css
  git commit -m "feat: add MemClean showcase website structure and OLED dark mode styling"
  ```

---

### Task 3: Interactive Visualizations & Live Demo Engine (`website/app.js`)

**Files:**
- Create: `website/app.js`
- Modify: `website/index.html` (script tag link)

**Interfaces:**
- Consumes: User interaction on interactive memory simulator, copy buttons, and terminal tab switchers.
- Produces: Dynamic RAM reduction animation, interactive memory bar updates, clipboard feedback toast, and tab switching.

- [ ] **Step 1: Implement Interactive Memory Optimizer Simulator**
  - Provide an interactive "Clean Memory Now" button in the simulator card.
  - When clicked:
    - Simulates the 5 purge tiers step-by-step with micro-latency status indicators:
      `[1/5] Purging Standby List...` -> `[2/5] Flushing Modified Pages...` -> `[3/5] Trimming System File Cache...` -> `[4/5] Trimming Background Working Sets...` -> `[5/5] Preserving Foreground Game...`
    - Animates the simulated RAM bar dropping from 82% (13.2 GB / 16 GB, Warning Red) to 38% (6.1 GB / 16 GB, Optimal Green).
    - Updates stats with exact freed MB counter (`+7,120 MB Freed`).
    - Provides a "Reset" or auto-restore button to replay the simulation.

- [ ] **Step 2: Implement One-Click Copy Buttons for CLI Snippets**
  - Add copy-to-clipboard handlers on terminal snippets (`curl`, `cargo install`, `memclean --status`, `memclean --clean`).
  - Display smooth "Copied!" tooltip feedback (fade in/out 200ms).

- [ ] **Step 3: Implement Terminal Command Switcher**
  - Interactive tabs (`--status`, `--clean`, `--aggressive`, `--autostart`) that switch the rendered terminal stdout dynamically.

- [ ] **Step 4: Verify in Browser**
  Verify interactions in browser (button clicks, animations, copy events).

- [ ] **Step 5: Commit**
  ```bash
  git add website/app.js website/index.html
  git commit -m "feat: add interactive memory simulation widget and terminal switcher"
  ```

---

### Task 4: Responsive Verification, Accessibility Audit & GitHub Pages Deployment

**Files:**
- Create: `.github/workflows/deploy-site.yml`
- Create: `website/README.md`

**Interfaces:**
- Consumes: Static `website/` directory files.
- Produces: Automated GitHub Pages deployment pipeline and hosting instructions.

- [ ] **Step 1: Add GitHub Actions Deployment Workflow**
  - Create `.github/workflows/deploy-site.yml` to automatically publish the `website/` folder to GitHub Pages on every push to `main`.

- [ ] **Step 2: Responsive & Accessibility Validation**
  - Check mobile viewport (375px) for horizontal overflow, touch target sizes (≥44px), and legible text contrast.
  - Ensure all clickable cards have `cursor: pointer`.
  - Check `prefers-reduced-motion` compliance.

- [ ] **Step 3: Commit and Push**
  ```bash
  git add .github/workflows/deploy-site.yml website/README.md
  git commit -m "ci: add GitHub Pages deployment workflow for MemClean website"
  git push origin main
  ```

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-10-30-memclean-website-and-readme.md`. Two execution options:

1. **Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Inline Execution** - Execute tasks in this session step-by-step with review checkpoints

Which approach would you like to take?