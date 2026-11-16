# Plan: Professional + Minimal Website Refinement (MemClean)

## Goal
Refactor existing static site into a professional, minimal UI — Swiss/Minimalism style per ui-ux-pro-max design-system output. No rebuild, no new build step, keep single HTML/CSS/JS architecture. Reuse `styles.css` tokens where they align; consolidate where they don't.

## Product & Context (from analysis)
- Product: Windows system utility / memory optimizer (native Rust Win32)
- Audience: power users, gamers, PC enthusiasts — technical, utilitarian
- Style: dark, minimal, professional, content-first, high-contrast
- Stack: static HTML + CSS + vanilla JS modules (no framework/build)
- Fonts: Inter + JetBrains Mono (already loaded)

## Design System Decisions (locked from ui-ux-pro-max `--design-system`)
- Style: Minimalism & Swiss Style — clean, spacious, functional, geometric, sans-serif, grid-based
- Palette: single accent (emerald #10B981 for CTA/telemetry), cyan secondary, rose destructive. Drop multi-accent "pastel" clutter where possible.
- Surfaces: obsidian canvas #080B10, elevated surface #0E131F, hairline borders rgba(255,255,255,0.08–0.12)
- Motion: subtle hover 200–250ms only; respect prefers-reduced-motion (already present)
- Spacing: 4/8px scale already defined (--space-xs … --space-5xl)
- Typography: base 16px, Inter body, JetBrains Mono code/numbers, line-height 1.6

## Scope (7 pages)
`index.html`, `cli-reference.html`, `troubleshooting.html`, `changelog.html`, `roadmap.html`, `privacy.html`, `screenshots.html` + `styles.css`

Changes per page:
1. Header/nav: slim to essential links; sticky not required; clear focus rings
2. Hero: single headline + one CTA + one quick command; remove parallax/clutter
3. Feature cards: replace bento-grid clutter with 3–5 clean Swiss-grid cards, consistent radius/stroke
4. Tables/grids: fixed column widths, hairline borders, mono code cells, zebble not needed
5. FAQ: keep accordion, ensure focus-visible + 44px touch targets
6. Footer: consolidate link columns, MIT + copyright only
7. Remove emoji-as-icons; SVG only (already SVG brand mark — keep)

## styles.css changes
- Keep token block; deprecate duplicate `--pastel-*` aliases in favor of accent tokens
- Add `.card` base (border, radius, padding, hover 200ms, no layout shift)
- Add `.btn` focus-visible ring (2–4px), hover transform none (no layout shift)
- Add `.table` standard (hairline borders, mono code, left-aligned)
- Add `.reveal-on-scroll` already present; keep, gate on reduced-motion
- Mobile: 375px / 768px / 1024px / 1440px breakpoints already present; verify no horizontal scroll

## Accessibility / Pre-Delivery (ui-ux-pro-max pro-rules.md checklist)
- [ ] No emoji icons (SVG only)
- [ ] cursor-pointer on clickables
- [ ] Hover states 150–300ms smooth
- [ ] Focus rings visible for keyboard nav
- [ ] prefers-reduced-motion respected
- [ ] Text contrast 4.5:1 normal / 3:1 large
- [ ] Touch targets ≥44×44px, 8px+ spacing
- [ ] Responsive 375/768/1024/1440, no horizontal scroll
- [ ] Dark-mode contrast checked independently
- [ ] Sequential heading hierarchy h1→h6

## Verification
- Open all 7 HTML files locally; confirm no console errors, div-balanced
- Visual check at 375px, 768px, 1024px, 1440px
- Reduced-motion toggle: animations disabled
- Keyboard tab order through nav + CTA + footer links
- Commit + push `master` only after approval & green check

## Order of Work
1. Token consolidation + `.card`/`.btn`/`.table` base in styles.css
2. index.html hero + feature cards + FAQ polish
3. Remaining 6 pages: header/footer/table/FAQ consistency
4. Accessibility pass + responsive spot-check
5. Commit & push `master`

## Stop Point
Implementation begins only after you approve this plan. Reply with changes or "go."
