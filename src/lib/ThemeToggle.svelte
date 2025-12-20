<script lang="ts">
  import { onMount } from 'svelte';

  let theme = $state<'dark' | 'light'>('dark');
  let mounted = $state(false);

  onMount(() => {
    // Check localStorage first, then system preference
    const saved = localStorage.getItem('theme');
    if (saved === 'light' || saved === 'dark') {
      theme = saved;
    } else if (window.matchMedia('(prefers-color-scheme: light)').matches) {
      theme = 'light';
    }
    applyTheme(theme);
    mounted = true;
  });

  function applyTheme(newTheme: 'dark' | 'light') {
    document.documentElement.setAttribute('data-theme', newTheme);
    localStorage.setItem('theme', newTheme);
  }

  function toggleTheme() {
    theme = theme === 'dark' ? 'light' : 'dark';
    applyTheme(theme);
  }
</script>

{#if mounted}
  <button
    class="theme-toggle"
    onclick={toggleTheme}
    title={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
    aria-label={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
  >
    {#if theme === 'dark'}
      <!-- Sun icon -->
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="5"/>
        <line x1="12" y1="1" x2="12" y2="3"/>
        <line x1="12" y1="21" x2="12" y2="23"/>
        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
        <line x1="1" y1="12" x2="3" y2="12"/>
        <line x1="21" y1="12" x2="23" y2="12"/>
        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
      </svg>
    {:else}
      <!-- Moon icon -->
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
      </svg>
    {/if}
  </button>
{/if}

<style>
  .theme-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    padding: 0;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--blur-sm));
    -webkit-backdrop-filter: blur(var(--blur-sm));
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    color: var(--text-secondary);
    transition: all var(--transition-fast);
  }

  .theme-toggle:hover {
    background: var(--glass-bg-hover);
    color: var(--accent-color);
    border-color: var(--glass-border-light);
    transform: scale(1.05);
    box-shadow: 0 0 16px var(--accent-glow);
  }

  .theme-toggle:active {
    transform: scale(0.98);
  }

  .theme-toggle svg {
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .theme-toggle:hover svg {
    transform: rotate(15deg);
  }
</style>
