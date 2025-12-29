<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import CommitItem from './CommitItem.svelte';
  import GitGraphLine from './GitGraphLine.svelte';
  import { calculateGraphLayout } from './graphUtils';
  import type { CommitInfo, CommitPage } from './types';

  interface Props {
    repoPath: string;
    onEditCommit: (commit: CommitInfo) => void;
    onViewDiff: (commit: CommitInfo) => void;
    selectedCommits: Set<string>;
    onSelectionChange: (commits: Set<string>) => void;
  }

  let { repoPath, onEditCommit, onViewDiff, selectedCommits, onSelectionChange }: Props = $props();

  let commits: CommitInfo[] = $state([]);
  let loading = $state(false);
  let hasMore = $state(true);
  let totalCount = $state(0);
  let error = $state<string | null>(null);
  let searchQuery = $state('');
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;
  let currentRepoPath = $state('');

  const PAGE_SIZE = 50;
  const ROW_HEIGHT = 52;

  // Calculate graph layout whenever commits change
  let graphLayout = $derived(calculateGraphLayout(commits));

  async function loadCommits(reset = false, query = '') {
    console.log('[loadCommits]', { reset, query, loading, hasMore, currentRepoPath, commitsLen: commits.length });

    if (loading) {
      console.log('[loadCommits] already loading, skip');
      return;
    }
    if (!reset && !hasMore) {
      console.log('[loadCommits] no more to load');
      return;
    }
    if (!currentRepoPath) {
      console.log('[loadCommits] no repo path');
      return;
    }

    loading = true;
    error = null;

    try {
      const offset = reset ? 0 : commits.length;
      console.log('[loadCommits] fetching', { offset, limit: PAGE_SIZE, query: query.trim() || '(none)' });

      let result: CommitPage;
      if (query.trim()) {
        result = await invoke('search_commits', {
          repoPath: currentRepoPath,
          query: query.trim(),
          offset,
          limit: PAGE_SIZE
        });
      } else {
        result = await invoke('get_commits', {
          repoPath: currentRepoPath,
          offset,
          limit: PAGE_SIZE
        });
      }

      console.log('[loadCommits] result', { count: result.commits.length, hasMore: result.has_more, total: result.total_count });

      if (reset) {
        commits = result.commits;
      } else {
        commits = [...commits, ...result.commits];
      }
      hasMore = result.has_more;
      totalCount = result.total_count;
    } catch (e) {
      error = String(e);
      console.error('[loadCommits] error:', e);
    } finally {
      loading = false;
    }
  }

  function handleSearchInput(e: Event) {
    const target = e.target as HTMLInputElement;
    const newValue = target.value;
    console.log('[handleSearchInput]', newValue);
    searchQuery = newValue;

    // Debounce search
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }
    searchTimeout = setTimeout(() => {
      console.log('[handleSearchInput] debounce trigger', newValue);
      loadCommits(true, newValue);
    }, 300);
  }

  function clearSearch() {
    searchQuery = '';
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }
    loadCommits(true, '');
  }

  function handleScroll(e: Event) {
    const target = e.target as HTMLDivElement;
    const { scrollTop, scrollHeight, clientHeight } = target;
    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;

    // Load more when 200px from bottom
    if (distanceFromBottom < 200 && !loading && hasMore) {
      console.log('[handleScroll] near bottom, loading more', { distanceFromBottom, loading, hasMore });
      loadCommits(false, searchQuery);
    }
  }

  function handleSelect(hash: string, selected: boolean) {
    const newSelection = new Set(selectedCommits);
    if (selected) {
      newSelection.add(hash);
    } else {
      newSelection.delete(hash);
    }
    onSelectionChange(newSelection);
  }

  function selectAll() {
    const newSelection = new Set(commits.map(c => c.hash));
    onSelectionChange(newSelection);
  }

  function clearSelection() {
    onSelectionChange(new Set());
  }

  // Reload when repoPath changes
  $effect(() => {
    console.log('[effect] repoPath changed?', { repoPath, currentRepoPath });
    if (repoPath && repoPath !== currentRepoPath) {
      console.log('[effect] loading commits for new repo');
      currentRepoPath = repoPath;
      searchQuery = '';
      hasMore = true;
      commits = [];
      loadCommits(true, '');
    }
  });

  export function refresh() {
    loadCommits(true, searchQuery);
  }
</script>

<div class="commit-list-container">
  <div class="list-header">
    <div class="search-bar">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <path d="m21 21-4.35-4.35"/>
      </svg>
      <input
        type="text"
        class="search-input"
        placeholder="Search commits..."
        value={searchQuery}
        oninput={handleSearchInput}
      />
      {#if searchQuery}
        <button class="clear-search-btn" onclick={clearSearch} title="Clear search">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      {/if}
    </div>
    <div class="header-right">
      <span class="count">{totalCount} {searchQuery ? 'matches' : 'commits'}</span>
      <div class="selection-actions">
        {#if selectedCommits.size > 0}
          <span class="selected-count">{selectedCommits.size} selected</span>
          <button class="text-btn" onclick={clearSelection}>Clear</button>
        {:else}
          <button class="text-btn" onclick={selectAll}>Select all</button>
        {/if}
      </div>
    </div>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div
    class="commit-list"
    onscroll={handleScroll}
  >
    {#each commits as commit (commit.hash)}
      {@const graphNode = graphLayout.nodes.get(commit.hash)}
      <div class="commit-row">
        {#if graphNode && !searchQuery}
          <GitGraphLine
            node={graphNode}
            maxRails={graphLayout.maxRails}
            rowHeight={ROW_HEIGHT}
            isSelected={selectedCommits.has(commit.hash)}
          />
        {/if}
        <CommitItem
          {commit}
          selected={selectedCommits.has(commit.hash)}
          onSelect={handleSelect}
          onEdit={onEditCommit}
          {onViewDiff}
        />
      </div>
    {/each}

    {#if loading}
      <div class="loading">Loading...</div>
    {/if}

    {#if !loading && commits.length === 0 && !error}
      <div class="empty">
        {#if searchQuery}
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="11" cy="11" r="8"/>
            <path d="m21 21-4.35-4.35"/>
          </svg>
          <p>No commits matching "<strong>{searchQuery}</strong>"</p>
          <button class="text-btn" onclick={clearSearch}>Clear search</button>
        {:else}
          No commits found
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .commit-list-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--blur-lg));
    -webkit-backdrop-filter: blur(var(--blur-lg));
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-xl);
    box-shadow:
      var(--shadow-inner),
      var(--shadow-md);
  }

  .list-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    padding: 14px 18px;
    border-bottom: 1px solid var(--glass-border);
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.04) 0%, transparent 100%);
    border-radius: var(--radius-xl) var(--radius-xl) 0 0;
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    max-width: 400px;
    background: rgba(0, 0, 0, 0.25);
    backdrop-filter: blur(var(--blur-sm));
    -webkit-backdrop-filter: blur(var(--blur-sm));
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    padding: 0 14px;
    transition: all var(--transition-normal);
  }

  .search-bar:focus-within {
    border-color: var(--accent-color);
    box-shadow:
      0 0 0 3px var(--accent-muted),
      0 0 20px var(--accent-glow);
    background: rgba(0, 0, 0, 0.35);
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
    padding: 10px 0 !important;
    font-size: 14px;
    color: var(--text-color);
    width: 100%;
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

  .clear-search-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px;
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--muted-color);
    transition: all var(--transition-fast);
  }

  .clear-search-btn:hover {
    background: var(--glass-bg-hover);
    color: var(--text-color);
    border-color: var(--glass-border-light);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-shrink: 0;
  }

  .count {
    font-size: 13px;
    color: var(--muted-color);
    font-weight: 500;
    white-space: nowrap;
  }

  .selection-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .selected-count {
    font-size: 13px;
    font-weight: 600;
    color: var(--accent-hover);
    padding: 6px 12px;
    background: linear-gradient(135deg, var(--accent-muted) 0%, rgba(139, 92, 246, 0.15) 100%);
    border: 1px solid rgba(99, 102, 241, 0.25);
    border-radius: var(--radius-md);
    box-shadow: 0 0 12px var(--accent-glow);
  }

  .commit-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.1) transparent;
    padding: 8px;
    width: 100%;
  }

  .commit-row {
    display: flex;
    align-items: center;
    min-height: 52px;
    width: 100%;
  }

  .commit-row :global(.commit-item) {
    flex: 1;
    min-width: 0;
  }

  .commit-list::-webkit-scrollbar {
    width: 6px;
  }

  .commit-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .commit-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }

  .commit-list::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .loading, .empty {
    padding: 60px 24px;
    text-align: center;
    color: var(--muted-color);
    font-size: 14px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }

  .empty svg {
    opacity: 0.4;
    filter: drop-shadow(0 0 8px var(--accent-glow));
  }

  .empty p {
    margin: 0;
  }

  .empty strong {
    color: var(--text-secondary);
  }

  .error {
    padding: 16px 20px;
    text-align: center;
    color: var(--error-color);
    background: linear-gradient(135deg, var(--error-bg) 0%, rgba(248, 113, 113, 0.05) 100%);
    margin: 12px;
    border-radius: var(--radius-md);
    border: 1px solid rgba(248, 113, 113, 0.25);
    font-size: 13px;
    backdrop-filter: blur(var(--blur-sm));
    -webkit-backdrop-filter: blur(var(--blur-sm));
  }

  /* ============================================
     Responsive Styles
     ============================================ */

  /* Tablet (< 768px) */
  @media (max-width: 768px) {
    .commit-list-container {
      border-radius: var(--radius-lg);
    }

    .list-header {
      flex-direction: column;
      align-items: stretch;
      gap: 12px;
      padding: 12px;
      border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    }

    .search-bar {
      max-width: none;
    }

    .header-right {
      justify-content: space-between;
    }

    .commit-list {
      padding: 6px;
    }
  }

  /* Mobile (< 480px) */
  @media (max-width: 480px) {
    .commit-list-container {
      border-radius: var(--radius-md);
    }

    .list-header {
      padding: 10px;
      gap: 10px;
      border-radius: var(--radius-md) var(--radius-md) 0 0;
    }

    .search-bar {
      padding: 0 10px;
      border-radius: var(--radius-md);
    }

    .search-input {
      font-size: 16px; /* Prevents zoom on iOS */
      padding: 8px 0;
    }

    .header-right {
      flex-wrap: wrap;
      gap: 8px;
    }

    .count {
      font-size: 12px;
    }

    .selection-actions {
      gap: 8px;
    }

    .selected-count {
      font-size: 12px;
      padding: 4px 8px;
    }

    .commit-list {
      padding: 4px;
    }

    .loading, .empty {
      padding: 40px 16px;
    }

    .empty svg {
      width: 36px;
      height: 36px;
    }

    .error {
      margin: 8px;
      padding: 12px;
      font-size: 12px;
    }
  }
</style>
