<script lang="ts">
  import type { CommitDiff, FileDiff } from './types';

  interface Props {
    diff: CommitDiff;
  }

  let { diff }: Props = $props();
  let expandedFiles = $state<Set<string>>(new Set());

  $effect(() => {
    expandedFiles = new Set(diff.files.map(f => f.path));
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
</script>

<div class="diff-viewer">
  <div class="diff-stats">
    <span class="stat files">{diff.stats.files_changed} file{diff.stats.files_changed !== 1 ? 's' : ''}</span>
    <span class="stat additions">+{diff.stats.insertions}</span>
    <span class="stat deletions">-{diff.stats.deletions}</span>
  </div>

  <div class="files-list">
    {#each diff.files as file}
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
                {#each file.lines as line}
                  {#if line.line_type === 'header'}
                    <tr class="hunk-header">
                      <td class="line-no"></td>
                      <td class="line-no"></td>
                      <td class="line-content">{line.content}</td>
                    </tr>
                  {:else}
                    <tr class="diff-line {line.line_type}">
                      <td class="line-no old">{line.old_line_no ?? ''}</td>
                      <td class="line-no new">{line.new_line_no ?? ''}</td>
                      <td class="line-content">
                        <span class="line-prefix">{line.line_type === 'add' ? '+' : line.line_type === 'delete' ? '-' : ' '}</span>
                        <span class="line-text">{line.content}</span>
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

  .diff-stats {
    display: flex;
    gap: 16px;
    padding: 14px 18px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--blur-sm));
    -webkit-backdrop-filter: blur(var(--blur-sm));
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    font-size: 13px;
    font-weight: 500;
    box-shadow: var(--shadow-inner);
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
    background: var(--glass-bg);
    backdrop-filter: blur(var(--blur-sm));
    -webkit-backdrop-filter: blur(var(--blur-sm));
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: var(--shadow-inner);
  }

  .file-section {
    border-bottom: 1px solid var(--glass-border);
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
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.03) 0%, transparent 100%);
    border: none;
    cursor: pointer;
    text-align: left;
    color: var(--text-color);
    font-size: 13px;
    transition: all var(--transition-fast);
  }

  .file-header:hover {
    background: var(--glass-bg-hover);
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
    background: rgba(0, 0, 0, 0.2);
    overflow-x: auto;
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
    background: linear-gradient(135deg, var(--accent-muted) 0%, rgba(139, 92, 246, 0.1) 100%);
    color: var(--accent-hover);
    font-weight: 500;
    border-bottom: 1px solid var(--glass-border);
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
    background: rgba(0, 0, 0, 0.15);
    user-select: none;
    border-right: 1px solid var(--glass-border);
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
    background: rgba(74, 222, 128, 0.1);
  }

  .diff-line.add .line-content {
    color: var(--success-color);
  }

  .diff-line.add .line-no {
    background: rgba(74, 222, 128, 0.08);
  }

  .diff-line.delete {
    background: rgba(248, 113, 113, 0.1);
  }

  .diff-line.delete .line-content {
    color: var(--error-color);
  }

  .diff-line.delete .line-no {
    background: rgba(248, 113, 113, 0.08);
  }

  .diff-line.context .line-content {
    color: var(--text-secondary);
  }

  /* ============================================
     Responsive Styles
     ============================================ */

  /* Tablet (< 768px) */
  @media (max-width: 768px) {
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
    .diff-stats {
      flex-wrap: wrap;
      padding: 10px 12px;
      gap: 8px;
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
