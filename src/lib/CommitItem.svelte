<script lang="ts">
  import type { CommitInfo } from './types';

  interface Props {
    commit: CommitInfo;
    selected: boolean;
    onSelect: (hash: string, selected: boolean) => void;
    onEdit: (commit: CommitInfo) => void;
    onViewDiff: (commit: CommitInfo) => void;
  }

  let { commit, selected, onSelect, onEdit, onViewDiff }: Props = $props();

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function getFirstLine(message: string): string {
    return message.split('\n')[0];
  }

  function handleCheckbox(e: Event) {
    const target = e.target as HTMLInputElement;
    onSelect(commit.hash, target.checked);
  }
</script>

<div class="commit-item" class:selected>
  <label class="checkbox-container">
    <input
      type="checkbox"
      checked={selected}
      onchange={handleCheckbox}
    />
    <span class="checkmark"></span>
  </label>

  <div class="commit-info">
    <div class="commit-header">
      <span class="hash">{commit.short_hash}</span>
      <span class="message" title={commit.message}>{getFirstLine(commit.message)}</span>
    </div>
    <div class="commit-meta">
      <span class="author">{commit.author}</span>
      <span class="date">{formatDate(commit.date)}</span>
    </div>
  </div>

  <div class="action-buttons">
    <button class="action-btn" onclick={() => onViewDiff(commit)} title="View diff">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <line x1="9" y1="15" x2="15" y2="15"/>
      </svg>
    </button>
    <button class="action-btn" onclick={() => onEdit(commit)} title="Edit message">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .commit-item {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 16px;
    margin-bottom: 4px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    transition: all var(--transition-normal);
  }

  .commit-item:hover {
    background: var(--glass-bg-hover);
    border-color: var(--glass-border);
  }

  .commit-item.selected {
    background: linear-gradient(135deg, var(--selected-bg) 0%, rgba(139, 92, 246, 0.08) 100%);
    border-color: rgba(99, 102, 241, 0.3);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.05),
      0 0 16px var(--accent-glow);
  }

  /* Custom checkbox */
  .checkbox-container {
    position: relative;
    cursor: pointer;
    user-select: none;
    flex-shrink: 0;
  }

  .checkbox-container input {
    position: absolute;
    opacity: 0;
    cursor: pointer;
    height: 0;
    width: 0;
  }

  .checkmark {
    display: block;
    height: 20px;
    width: 20px;
    background: rgba(0, 0, 0, 0.3);
    border: 2px solid var(--glass-border-light);
    border-radius: var(--radius-xs);
    transition: all var(--transition-fast);
  }

  .checkbox-container:hover input ~ .checkmark {
    border-color: var(--accent-color);
    background: var(--accent-muted);
    box-shadow: 0 0 12px var(--accent-glow);
  }

  .checkbox-container input:checked ~ .checkmark {
    background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
    border-color: transparent;
    box-shadow:
      0 0 16px var(--accent-glow),
      inset 0 1px 0 rgba(255, 255, 255, 0.2);
  }

  .checkmark:after {
    content: "";
    position: absolute;
    display: none;
    left: 7px;
    top: 3px;
    width: 5px;
    height: 10px;
    border: solid white;
    border-width: 0 2px 2px 0;
    transform: rotate(45deg);
  }

  .checkbox-container input:checked ~ .checkmark:after {
    display: block;
  }

  /* Commit info */
  .commit-info {
    flex: 1;
    min-width: 0;
  }

  .commit-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 6px;
  }

  .commit-header :global(.hash) {
    flex-shrink: 0;
  }

  .message {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-color);
  }

  .commit-meta {
    font-size: 12px;
    color: var(--muted-color);
    display: flex;
    gap: 16px;
  }

  .commit-meta .author {
    color: var(--text-secondary);
  }

  /* Action buttons */
  .action-buttons {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .commit-item:hover .action-buttons {
    opacity: 1;
  }

  .action-btn {
    padding: 8px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--blur-sm));
    -webkit-backdrop-filter: blur(var(--blur-sm));
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--muted-color);
    transition: all var(--transition-fast);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .action-btn:hover {
    background: var(--accent-muted);
    color: var(--accent-hover);
    border-color: rgba(99, 102, 241, 0.4);
    box-shadow: 0 0 12px var(--accent-glow);
    transform: translateY(-1px);
  }

  /* ============================================
     Responsive Styles
     ============================================ */

  /* Tablet (< 768px) */
  @media (max-width: 768px) {
    .commit-item {
      padding: 12px;
      gap: 12px;
    }

    /* Always show action buttons on touch devices */
    .action-buttons {
      opacity: 1;
    }

    .action-btn {
      padding: 10px;
    }
  }

  /* Mobile (< 480px) */
  @media (max-width: 480px) {
    .commit-item {
      flex-wrap: wrap;
      padding: 10px;
      gap: 8px;
    }

    .checkbox-container {
      order: 1;
    }

    .commit-info {
      order: 2;
      flex: 1;
      min-width: calc(100% - 40px);
    }

    .action-buttons {
      order: 3;
      width: 100%;
      justify-content: flex-end;
      margin-top: 4px;
      padding-top: 8px;
      border-top: 1px solid var(--glass-border);
    }

    .commit-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 6px;
    }

    .message {
      font-size: 13px;
      white-space: normal;
      line-height: 1.4;
    }

    .commit-meta {
      flex-wrap: wrap;
      gap: 8px;
      font-size: 11px;
    }

    .checkmark {
      height: 22px;
      width: 22px;
    }

    .checkmark:after {
      left: 8px;
      top: 4px;
    }
  }
</style>
