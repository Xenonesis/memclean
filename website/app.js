/**
 * MemClean — Utilitarian Website Engine Bootstrap
 * Zero-dependency pure ES6 modular JavaScript architecture.
 *
 * Modules:
 * - ./js/modules/simulator.js: 5-Tier NT Kernel Memory Optimizer Simulator
 * - ./js/modules/terminal.js: Scriptable CLI Terminal Command Switcher
 * - ./js/modules/interactions.js: Universal Clipboard, FAQ, Scroll Reveals, Mobile Nav & Clock
 */

'use strict';

import { initMemorySimulator } from './js/modules/simulator.js';
import { initTerminalShowcase } from './js/modules/terminal.js';
import {
  initClipboardHandlers,
  initFaqAccordion,
  initScrollReveals,
  initMobileNav,
  initSmoothScroll,
  initTaskbarClock,
} from './js/modules/interactions.js';

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
