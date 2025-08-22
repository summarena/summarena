// Umami analytics tracking functionality
// This module provides click tracking, scroll tracking, and CTA tracking

interface UmamiGlobal {
  track: (eventName: string, eventData?: Record<string, string>) => void;
}

// Extend Window interface for umami
declare global {
  interface Window {
    umami?: UmamiGlobal;
  }
}

// This makes the file a module
export {};

/**
 * Initialize click tracking for elements with data-track attribute
 * and automatically track CTA buttons and external links
 */
function initClickTracking(): void {
  // Track elements with explicit data-track attribute
  const trackableElements = document.querySelectorAll<HTMLElement>('[data-track]');

  trackableElements.forEach(element => {
    element.addEventListener('click', function (this: HTMLElement) {
      const trackingName = this.getAttribute('data-track');
      if (window.umami && trackingName) {
        window.umami.track('click', { element: trackingName });
      }
    });
  });

  // Track all CTA buttons and external links
  const ctaButtons = document.querySelectorAll<HTMLElement>('button, [role="button"], .cta, .btn');
  const externalLinks = document.querySelectorAll<HTMLAnchorElement>(
    `a[href^="http"]:not([href*="${window.location.hostname}"])`
  );

  ctaButtons.forEach(button => {
    if (!button.hasAttribute('data-track')) {
      button.addEventListener('click', function (this: HTMLElement) {
        const buttonText = this.textContent?.trim() || this.getAttribute('aria-label') || 'button';
        if (window.umami) {
          window.umami.track('cta-click', { button: buttonText });
        }
      });
    }
  });

  externalLinks.forEach(link => {
    if (!link.hasAttribute('data-track')) {
      link.addEventListener('click', function (this: HTMLAnchorElement) {
        const href = this.getAttribute('href');
        const linkText = this.textContent?.trim() || href || 'external-link';
        if (window.umami && href) {
          window.umami.track('external-link', { url: href, text: linkText });
        }
      });
    }
  });
}

/**
 * Initialize scroll tracking with intersection observer for elements with data-track-view attribute
 * Only tracks once per page load and ignores elements in first viewport
 */
function initScrollTracking(): void {
  const trackedSections = new Set<string>();
  const trackableSections = document.querySelectorAll<HTMLElement>('[data-track-view]');

  if (trackableSections.length === 0) return;

  const observer = new IntersectionObserver(
    entries => {
      entries.forEach(entry => {
        if (entry.isIntersecting && entry.intersectionRatio >= 0.5) {
          const sectionName = entry.target.getAttribute('data-track-view');

          if (sectionName && !trackedSections.has(sectionName)) {
            // Check if element is not in the first viewport
            const rect = entry.target.getBoundingClientRect();
            const isInFirstViewport = rect.top < window.innerHeight && rect.top >= 0;

            if (!isInFirstViewport || window.pageYOffset > 0) {
              trackedSections.add(sectionName);
              if (window.umami) {
                window.umami.track('section-view', { section: sectionName });
              }
            }
          }
        }
      });
    },
    {
      threshold: 0.5,
      rootMargin: '0px 0px -20% 0px',
    }
  );

  trackableSections.forEach(section => {
    observer.observe(section);
  });
}

/**
 * Initialize all analytics tracking when DOM is loaded
 */
function initAnalytics(): void {
  initClickTracking();
  initScrollTracking();
}

// Initialize analytics when DOM is loaded
document.addEventListener('DOMContentLoaded', initAnalytics);

// Also initialize immediately if DOM is already loaded (for dynamic imports/test environments)
if (document.readyState === 'loading') {
  // Document is still loading, wait for DOMContentLoaded
} else {
  // Document has already loaded, initialize immediately
  initAnalytics();
}
