/**
 * MemClean — Utilitarian Interactions & UI Enhancements Component
 */

'use strict';

let toastTimeout = null;

export function showToast(message = 'Copied to clipboard') {
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

export function copyToClipboard(text, triggerBtn = null) {
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

export function initClipboardHandlers() {
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

export function initFaqAccordion() {
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

export function initScrollReveals() {
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

export function initMobileNav() {
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

export function initSmoothScroll() {
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

export function initTaskbarClock() {
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
