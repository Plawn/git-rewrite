/**
 * Centralized constants for the application.
 * Keeping these in one place makes it easy to adjust behavior.
 */

/** Pagination settings */
export const PAGINATION = {
  /** Number of commits to load per page */
  PAGE_SIZE: 50,
  /** Distance from bottom (px) to trigger loading more */
  SCROLL_THRESHOLD: 200,
} as const;

/** UI dimensions */
export const UI = {
  /** Height of each commit row in pixels */
  ROW_HEIGHT: 52,
} as const;

/** Timing settings (milliseconds) */
export const TIMING = {
  /** Debounce delay for search input */
  SEARCH_DEBOUNCE_MS: 300,
  /** Delay before scrolling to match in diff viewer */
  SCROLL_TO_MATCH_DELAY_MS: 50,
} as const;

/** Git graph visualization */
export const GRAPH = {
  /** Number of colors in the branch color rotation */
  COLOR_COUNT: 8,
} as const;

/** localStorage keys */
export const STORAGE_KEYS = {
  THEME: 'git-rewrite-theme',
} as const;
