# Plan: Website Detail Expansion (memclean)

**Goal:** `website/` ko detailed banao — naye standalone pages + existing index.html sections ko expand karo taaki user ko full technical clarity mile.

**Status:** Draft — approval pending
**Scope:** Static HTML/CSS/JS only (no build step, no framework). Reuse existing design tokens from `styles.css`.

---

## Current State (baseline)
- `website/index.html` — 1271 lines: hero, simulator, features bento, architecture 5-tier, comparison table, CLI terminal, FAQ, download, footer.
- `website/styles.css` — 1720 lines, design tokens already defined.
- `website/js/modules/{interactions,simulator,terminal}.js` — existing interactivity.
- `docs/` — screenshots + superpowers plans exist.

---

## Phase 1: New Standalone Pages (add to `website/`)

| File | Purpose | Key Content |
|------|---------|-------------|
| `cli-reference.html` | Full CLI command reference | `--status --clean --network --vram --cpu --telemetry --hud --json --timer-res`, exit codes, output schema examples |
| `troubleshooting.html` | Common issues & fixes | UAC elevation fail, admin privileges (SeProfileSingleProcessPrivilege), SSD vs HDD behavior, config reset location (`%APPDATA%\MemClean\config.toml`) |
| `changelog.html` | Release history | v1.1.3, v1.1.2, v1.1.1 with dates, fixes, features |
| `roadmap.html` | Future features | Upcoming phases (network stack, VRAM, CPU affinity, hard-fault telemetry, game HUD) with status tags |
| `privacy.html` | Privacy / security statement | Zero telemetry, no network calls, local-only config, open-source MIT |
| `screenshots.html` | UI gallery | Tray icon, settings dialog, main window, simulator preview (reuse `docs/assets/*.png`) |

**Nav:** Add these to header `<ul class="nav-links">` and footer nav columns on all pages (including index.html).

---

## Phase 2: Expand Existing index.html Sections

### A. New "Benchmarks & Performance" section (before comparison)
- Canvas/JS bar chart: idle RAM footprint vs competitors (MemClean vs Sordum vs generic).
- Table: boot time impact, CPU% at idle, binary size, purge latency (ms).
- Source note: "Measured on Windows 11 23H2, 16 GB DDR4."

### B. System Requirements table
- OS: Windows 10 / 11 (64-bit), version 1809+.
- Privileges: Administrator (auto-elevated via Task Scheduler).
- Disk: ~2.4 MB installer, ~1.1 MB portable.
- RAM: < 2 MB idle.
- Dependencies: None (zero external DLL, no .NET, no Electron).

### C. Expand CLI section
- Add `--aggressive`, `--set-threshold <n>`, `--apply-device-defaults`, `--export-config` stub commands with sample output.
- Show JSON output block example.

### D. Add more FAQ items
- "What does 'Best for Your Device' do?" (SSD vs HDD defaults)
- "Where is config stored?"
- "Does it work on HDD?"
- "Is it anti-cheat safe?" (zero-injection HUD note)

### E. Expand Features bento
- Add cards for: "Best for Your Device (SSD/HDD auto-detect)", "Threshold Auto-Clean (v1.1.2 fix)", "Owner-Draw Hover Feedback (v1.1.2 polish)".

---

## Phase 3: CSS/JS Polish
- Smooth scroll behavior for anchor links.
- Mobile hamburger menu already exists — verify touch targets.
- Add `prefers-reduced-motion` respect for reveal-on-scroll animations.
- Ensure all new pages link `styles.css` and `app.js` consistently.

---

## Acceptance Criteria
1. All 6 new pages exist and open without broken links.
2. Header/footer nav consistent across `index.html` + new pages.
3. `cargo`-independent — only HTML/CSS/JS changes.
4. Website builds/opens locally (double-click `index.html`).
5. Commit + push to `master` (separate from code changes).

---

## Execution Order (when approved)
1. Create `cli-reference.html`, `troubleshooting.html`, `changelog.html`, `roadmap.html`, `privacy.html`, `screenshots.html`.
2. Update `index.html` nav/footer + add benchmark + system-requirements + expanded CLI/FAQ/features.
3. Tweak `styles.css` for new page layouts if needed.
4. Verify all links open.
5. Commit + push `master`.
