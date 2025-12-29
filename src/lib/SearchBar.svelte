<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    value: string;
    placeholder?: string;
    onInput: (value: string) => void;
    onClear: () => void;
    onKeydown?: (e: KeyboardEvent) => void;
    extraContent?: Snippet;
  }

  let {
    value,
    placeholder = 'Search...',
    onInput,
    onClear,
    onKeydown,
    extraContent
  }: Props = $props();

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    onInput(target.value);
  }
</script>

<div class="search-bar">
  <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
    <circle cx="11" cy="11" r="8"/>
    <path d="m21 21-4.35-4.35"/>
  </svg>
  <input
    type="text"
    class="search-input"
    {placeholder}
    {value}
    oninput={handleInput}
    onkeydown={onKeydown}
  />
  {#if extraContent}
    {@render extraContent()}
  {/if}
  {#if value}
    <button class="clear-btn" onclick={onClear} title="Clear search">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18"/>
        <line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
    </button>
  {/if}
</div>

<style>
  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    flex: 1;
    transition: all var(--transition-normal);
  }

  :global([data-theme="light"]) .search-bar {
    background: rgba(0, 0, 0, 0.03);
  }

  .search-bar:focus-within {
    border-color: var(--accent-color);
    box-shadow:
      0 0 0 3px var(--accent-muted),
      0 0 20px var(--accent-glow);
    background: rgba(0, 0, 0, 0.35);
  }

  :global([data-theme="light"]) .search-bar:focus-within {
    background: rgba(0, 0, 0, 0.05);
  }

  .search-icon {
    color: var(--muted-color);
    flex-shrink: 0;
    transition: color var(--transition-fast);
  }

  .search-bar:focus-within .search-icon {
    color: var(--accent-color);
  }

  .search-input {
    flex: 1;
    border: none !important;
    background: transparent !important;
    backdrop-filter: none !important;
    -webkit-backdrop-filter: none !important;
    padding: 4px 0 !important;
    font-size: 13px;
    color: var(--text-color);
    min-width: 80px;
    border-radius: 0 !important;
    box-shadow: none !important;
  }

  .search-input:focus {
    outline: none;
    box-shadow: none !important;
    border: none !important;
  }

  .search-input::placeholder {
    color: var(--muted-color);
  }

  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    background: transparent;
    border: none;
    border-radius: var(--radius-xs);
    cursor: pointer;
    color: var(--muted-color);
    transition: all var(--transition-fast);
  }

  .clear-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-color);
  }

  :global([data-theme="light"]) .clear-btn:hover {
    background: rgba(0, 0, 0, 0.08);
  }

  /* Mobile: prevent zoom on iOS */
  @media (max-width: 480px) {
    .search-input {
      font-size: 16px !important;
    }
  }
</style>
