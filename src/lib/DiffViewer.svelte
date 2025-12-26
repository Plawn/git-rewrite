<script lang="ts">
  import type { CommitDiff, FileDiff } from './types';

  interface Props {
    diff: CommitDiff;
  }

  let { diff }: Props = $props();
  let expandedFiles = $state<Set<string>>(new Set());
  let searchQuery = $state('');
  let currentMatchIndex = $state(0);
  let matchElements: HTMLElement[] = [];

  // Find all matches across all files
  let matches = $derived.by(() => {
    if (!searchQuery.trim()) return [];
    const query = searchQuery.toLowerCase();
    const results: { file: string; lineIndex: number }[] = [];

    for (const file of diff.files) {
      file.lines.forEach((line, lineIndex) => {
        if (line.content.toLowerCase().includes(query)) {
          results.push({ file: file.path, lineIndex });
        }
      });
    }
    return results;
  });

  let matchCount = $derived(matches.length);

  // Filter files to only show those with matches when searching
  let filteredFiles = $derived.by(() => {
    if (!searchQuery.trim()) return diff.files;
    const filesWithMatches = new Set(matches.map(m => m.file));
    return diff.files.filter(f => filesWithMatches.has(f.path));
  });

  $effect(() => {
    expandedFiles = new Set(diff.files.map(f => f.path));
  });

  // Reset match index when search changes
  $effect(() => {
    if (searchQuery) {
      currentMatchIndex = 0;
    }
  });

  function toggleFile(path: string) {
    if (expandedFiles.has(path)) {
      expandedFiles.delete(path);
    } else {
      expandedFiles.add(path);
    }
    expandedFiles = new Set(expandedFiles);
  }

  function getStatusClass(status: string): string {
    switch (status) {
      case 'Added': return 'status-added';
      case 'Deleted': return 'status-deleted';
      case 'Modified': return 'status-modified';
      case 'Renamed': return 'status-renamed';
      default: return '';
    }
  }

  function getStatusIcon(status: string): string {
    switch (status) {
      case 'Added': return '+';
      case 'Deleted': return '-';
      case 'Modified': return '~';
      case 'Renamed': return '→';
      default: return '?';
    }
  }

  function highlightText(text: string, query: string): string {
    if (!query.trim()) return text;
    const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(`(${escaped})`, 'gi');
    return text.replace(regex, '<mark class="search-highlight">$1</mark>');
  }

  function clearSearch() {
    searchQuery = '';
    currentMatchIndex = 0;
  }

  function goToNextMatch() {
    if (matchCount > 0) {
      currentMatchIndex = (currentMatchIndex + 1) % matchCount;
      scrollToMatch();
    }
  }

  function goToPrevMatch() {
    if (matchCount > 0) {
      currentMatchIndex = (currentMatchIndex - 1 + matchCount) % matchCount;
      scrollToMatch();
    }
  }

  function scrollToMatch() {
    // Ensure the file containing the match is expanded
    if (matches[currentMatchIndex]) {
      const match = matches[currentMatchIndex];
      if (!expandedFiles.has(match.file)) {
        expandedFiles.add(match.file);
        expandedFiles = new Set(expandedFiles);
      }
      // Scroll to match after DOM update
      setTimeout(() => {
        const elements = document.querySelectorAll('.search-match');
        const target = elements[currentMatchIndex] as HTMLElement;
        if (target) {
          target.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
      }, 50);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && searchQuery) {
      if (e.shiftKey) {
        goToPrevMatch();
      } else {
        goToNextMatch();
      }
      e.preventDefault();
    }
  }

  function isCurrentMatch(filePath: string, lineIndex: number): boolean {
    if (!matches[currentMatchIndex]) return false;
    return matches[currentMatchIndex].file === filePath &&
           matches[currentMatchIndex].lineIndex === lineIndex;
  }

  function isMatch(filePath: string, lineIndex: number): boolean {
    return matches.some(m => m.file === filePath && m.lineIndex === lineIndex);
  }
</script>

<div class="diff-viewer">
  <div class="diff-toolbar">
    <div class="diff-stats">
      <span class="stat files">{diff.stats.files_changed} file{diff.stats.files_changed !== 1 ? 's' : ''}</span>
      <span class="stat additions">+{diff.stats.insertions}</span>
      <span class="stat deletions">-{diff.stats.deletions}</span>
    </div>

    <div class="search-bar">
      <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <path d="m21 21-4.35-4.35"/>
      </svg>
      <input
        type="text"
        class="search-input"
        placeholder="Search in diff..."
        bind:value={searchQuery}
        onkeydown={handleKeydown}
      />
      {#if searchQuery}
        <span class="match-count" class:no-matches={matchCount === 0}>
          {#if matchCount > 0}
            {currentMatchIndex + 1}/{matchCount} in {filteredFiles.length} file{filteredFiles.length !== 1 ? 's' : ''}
          {:else}
            No matches
          {/if}
        </span>
        <div class="search-nav">
          <button class="nav-btn" onclick={goToPrevMatch} disabled={matchCount === 0} title="Previous match (Shift+Enter)">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="18 15 12 9 6 15"></polyline>
            </svg>
          </button>
          <button class="nav-btn" onclick={goToNextMatch} disabled={matchCount === 0} title="Next match (Enter)">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 9 12 15 18 9"></polyline>
            </svg>
          </button>
        </div>
        <button class="clear-btn" onclick={clearSearch} title="Clear search">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      {/if}
    </div>
  </div>

  <div class="files-list">
    {#each filteredFiles as file}
      <div class="file-section">
        <button class="file-header" onclick={() => toggleFile(file.path)}>
          <span class="expand-icon" class:expanded={expandedFiles.has(file.path)}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
          </span>
          <span class="status-badge {getStatusClass(file.status)}">{getStatusIcon(file.status)}</span>
          <span class="file-path">
            {#if file.old_path}
              <span class="old-path">{file.old_path}</span>
              <span class="arrow">→</span>
            {/if}
            {file.path}
          </span>
          <span class="file-stats">
            {#if file.insertions > 0}
              <span class="additions">+{file.insertions}</span>
            {/if}
            {#if file.deletions > 0}
              <span class="deletions">-{file.deletions}</span>
            {/if}
          </span>
        </button>

        {#if expandedFiles.has(file.path)}
          <div class="file-diff">
            <table class="diff-table">
              <tbody>
                {#each file.lines as line, lineIndex}
                  {#if line.line_type === 'header'}
                    <tr class="hunk-header">
                      <td class="line-no"></td>
                      <td class="line-no"></td>
                      <td class="line-content">{line.content}</td>
                    </tr>
                  {:else}
                    <tr
                      class="diff-line {line.line_type}"
                      class:search-match={isMatch(file.path, lineIndex)}
                      class:current-match={isCurrentMatch(file.path, lineIndex)}
                    >
                      <td class="line-no old">{line.old_line_no ?? ''}</td>
                      <td class="line-no new">{line.new_line_no ?? ''}</td>
                      <td class="line-content">
                        <span class="line-prefix">{line.line_type === 'add' ? '+' : line.line_type === 'delete' ? '-' : ' '}</span>
                        {#if searchQuery.trim()}
                          <span class="line-text">{@html highlightText(line.content, searchQuery)}</span>
                        {:else}
                          <span class="line-text">{line.content}</span>
                        {/if}
                      </td>
                    </tr>
                  {/if}
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .diff-viewer {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .diff-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .diff-stats {
    display: flex;
    gap: 16px;
    padding: 10px 14px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-md);
    font-size: 13px;
    font-weight: 500;
  }

  :global([data-theme="light"]) .diff-stats {
    background: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.08);
  }

  /* Search bar styles */
  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-md);
    flex: 1;
    max-width: 350px;
    transition: all var(--transition-fast);
  }

  :global([data-theme="light"]) .search-bar {
    background: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.08);
  }

  .search-bar:focus-within {
    border-color: var(--accent-color);
    box-shadow: 0 0 0 2px var(--accent-muted);
  }

  .search-icon {
    color: var(--muted-color);
    flex-shrink: 0;
  }

  .search-bar:focus-within .search-icon {
    color: var(--accent-color);
  }

  .search-input {
    flex: 1;
    border: none !important;
    background: transparent !important;
    padding: 4px 0 !important;
    font-size: 13px;
    color: var(--text-color);
    min-width: 100px;
    box-shadow: none !important;
    border-radius: 0 !important;
  }

  .search-input:focus {
    outline: none;
  }

  .search-input::placeholder {
    color: var(--muted-color);
  }

  .match-count {
    font-size: 11px;
    color: var(--text-secondary);
    padding: 2px 8px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-xs);
    white-space: nowrap;
  }

  .match-count.no-matches {
    color: var(--error-color);
    background: rgba(248, 113, 113, 0.15);
  }

  :global([data-theme="light"]) .match-count {
    background: rgba(0, 0, 0, 0.06);
  }

  :global([data-theme="light"]) .match-count.no-matches {
    background: rgba(248, 113, 113, 0.1);
  }

  .search-nav {
    display: flex;
    gap: 2px;
  }

  .nav-btn, .clear-btn {
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

  .nav-btn:hover:not(:disabled), .clear-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-color);
  }

  :global([data-theme="light"]) .nav-btn:hover:not(:disabled),
  :global([data-theme="light"]) .clear-btn:hover {
    background: rgba(0, 0, 0, 0.08);
  }

  .nav-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  /* Search highlight styles */
  :global(.search-highlight) {
    background: rgba(251, 191, 36, 0.4);
    color: inherit;
    padding: 1px 2px;
    border-radius: 2px;
  }

  :global([data-theme="light"] .search-highlight) {
    background: rgba(251, 191, 36, 0.5);
  }

  .diff-line.search-match {
    background: rgba(251, 191, 36, 0.1) !important;
  }

  .diff-line.search-match .line-no {
    background: rgba(251, 191, 36, 0.15) !important;
  }

  .diff-line.current-match {
    background: rgba(251, 191, 36, 0.25) !important;
    box-shadow: inset 3px 0 0 var(--warning-color);
  }

  .diff-line.current-match .line-no {
    background: rgba(251, 191, 36, 0.3) !important;
  }

  .stat.files {
    color: var(--text-secondary);
  }

  .stat.additions {
    color: var(--success-color);
    text-shadow: 0 0 12px rgba(74, 222, 128, 0.3);
  }

  .stat.deletions {
    color: var(--error-color);
    text-shadow: 0 0 12px rgba(248, 113, 113, 0.3);
  }

  .files-list {
    display: flex;
    flex-direction: column;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  :global([data-theme="light"]) .files-list {
    background: rgba(0, 0, 0, 0.02);
    border: 1px solid rgba(0, 0, 0, 0.08);
  }

  .file-section {
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  :global([data-theme="light"]) .file-section {
    border-bottom: 1px solid rgba(0, 0, 0, 0.06);
  }

  .file-section:last-child {
    border-bottom: none;
  }

  .file-header {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.02);
    border: none;
    cursor: pointer;
    text-align: left;
    color: var(--text-color);
    font-size: 13px;
    transition: all var(--transition-fast);
  }

  :global([data-theme="light"]) .file-header {
    background: rgba(0, 0, 0, 0.01);
  }

  .file-header:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  :global([data-theme="light"]) .file-header:hover {
    background: rgba(0, 0, 0, 0.04);
  }

  .expand-icon {
    color: var(--muted-color);
    display: flex;
    align-items: center;
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .expand-icon.expanded {
    transform: rotate(90deg);
  }

  .status-badge {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-xs);
    font-size: 12px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .status-added {
    background: linear-gradient(135deg, rgba(74, 222, 128, 0.2) 0%, rgba(74, 222, 128, 0.1) 100%);
    color: var(--success-color);
    border: 1px solid rgba(74, 222, 128, 0.3);
  }

  .status-deleted {
    background: linear-gradient(135deg, rgba(248, 113, 113, 0.2) 0%, rgba(248, 113, 113, 0.1) 100%);
    color: var(--error-color);
    border: 1px solid rgba(248, 113, 113, 0.3);
  }

  .status-modified {
    background: linear-gradient(135deg, rgba(251, 191, 36, 0.2) 0%, rgba(251, 191, 36, 0.1) 100%);
    color: var(--warning-color);
    border: 1px solid rgba(251, 191, 36, 0.3);
  }

  .status-renamed {
    background: linear-gradient(135deg, rgba(99, 102, 241, 0.2) 0%, rgba(99, 102, 241, 0.1) 100%);
    color: var(--accent-color);
    border: 1px solid rgba(99, 102, 241, 0.3);
  }

  .file-path {
    flex: 1;
    font-family: 'SF Mono', 'Fira Code', 'Monaco', monospace;
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .old-path {
    color: var(--muted-color);
    text-decoration: line-through;
  }

  .arrow {
    color: var(--muted-color);
    margin: 0 4px;
  }

  .file-stats {
    display: flex;
    gap: 10px;
    font-size: 12px;
    font-weight: 500;
  }

  .file-stats .additions {
    color: var(--success-color);
  }

  .file-stats .deletions {
    color: var(--error-color);
  }

  .file-diff {
    background: rgba(0, 0, 0, 0.15);
    overflow-x: auto;
  }

  :global([data-theme="light"]) .file-diff {
    background: rgba(0, 0, 0, 0.03);
  }

  .diff-table {
    width: 100%;
    border-collapse: collapse;
    font-family: 'SF Mono', 'Fira Code', 'Monaco', monospace;
    font-size: 12px;
    line-height: 1.6;
  }

  .hunk-header td {
    padding: 10px 14px;
    background: rgba(99, 102, 241, 0.15);
    color: var(--accent-hover);
    font-weight: 500;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  :global([data-theme="light"]) .hunk-header td {
    background: rgba(99, 102, 241, 0.1);
    border-bottom: 1px solid rgba(0, 0, 0, 0.06);
  }

  .diff-line td {
    padding: 0;
    vertical-align: top;
  }

  .line-no {
    width: 50px;
    min-width: 50px;
    padding: 2px 10px;
    text-align: right;
    color: var(--muted-color);
    background: rgba(0, 0, 0, 0.1);
    user-select: none;
    border-right: 1px solid rgba(255, 255, 255, 0.04);
  }

  :global([data-theme="light"]) .line-no {
    background: rgba(0, 0, 0, 0.04);
    border-right: 1px solid rgba(0, 0, 0, 0.06);
  }

  .line-no.old {
    border-right: none;
  }

  .line-content {
    padding: 2px 14px;
    white-space: pre;
  }

  .line-prefix {
    display: inline-block;
    width: 18px;
    color: inherit;
    user-select: none;
  }

  .diff-line.add {
    background: rgba(74, 222, 128, 0.12);
  }

  .diff-line.add .line-content {
    color: var(--success-color);
  }

  .diff-line.add .line-no {
    background: rgba(74, 222, 128, 0.1);
  }

  .diff-line.delete {
    background: rgba(248, 113, 113, 0.12);
  }

  .diff-line.delete .line-content {
    color: var(--error-color);
  }

  .diff-line.delete .line-no {
    background: rgba(248, 113, 113, 0.1);
  }

  .diff-line.context .line-content {
    color: var(--text-secondary);
  }

  /* ============================================
     Responsive Styles
     ============================================ */

  /* Tablet (< 768px) */
  @media (max-width: 768px) {
    .diff-toolbar {
      flex-direction: column;
      align-items: stretch;
    }

    .search-bar {
      max-width: none;
    }

    .diff-stats {
      padding: 12px 14px;
      gap: 12px;
      font-size: 12px;
    }

    .file-header {
      padding: 10px 12px;
      gap: 10px;
    }

    .file-path {
      font-size: 11px;
    }

    .diff-table {
      font-size: 11px;
    }

    .line-no {
      width: 40px;
      min-width: 40px;
      padding: 2px 6px;
    }

    .line-content {
      padding: 2px 10px;
    }
  }

  /* Mobile (< 480px) */
  @media (max-width: 480px) {
    .diff-toolbar {
      gap: 10px;
    }

    .diff-stats {
      flex-wrap: wrap;
      padding: 10px 12px;
      gap: 8px;
    }

    .search-bar {
      padding: 4px 10px;
      gap: 6px;
    }

    .search-input {
      font-size: 16px !important; /* Prevents zoom on iOS */
    }

    .match-count {
      font-size: 10px;
      padding: 2px 6px;
    }

    .file-header {
      padding: 8px 10px;
      gap: 8px;
    }

    .expand-icon svg {
      width: 10px;
      height: 10px;
    }

    .status-badge {
      width: 18px;
      height: 18px;
      font-size: 10px;
    }

    .file-path {
      font-size: 10px;
    }

    .file-stats {
      font-size: 10px;
      gap: 6px;
    }

    .diff-table {
      font-size: 10px;
      line-height: 1.4;
    }

    .hunk-header td {
      padding: 8px 10px;
      font-size: 10px;
    }

    .line-no {
      width: 32px;
      min-width: 32px;
      padding: 1px 4px;
      font-size: 9px;
    }

    .line-content {
      padding: 1px 8px;
    }

    .line-prefix {
      width: 14px;
    }
  }
</style>
