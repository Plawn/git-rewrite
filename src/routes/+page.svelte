<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import CommitList from '$lib/CommitList.svelte';
  import Modal from '$lib/Modal.svelte';
  import DiffViewer from '$lib/DiffViewer.svelte';
  import ThemeToggle from '$lib/ThemeToggle.svelte';
  import { createModalState, createAsyncModalState } from '$lib/modalState.svelte';
  import type { CommitInfo, RepoInfo, BranchInfo, CommitDiff, RepoValidation } from '$lib/types';

  let repoPath = $state<string | null>(null);
  let repoInfo = $state<RepoInfo | null>(null);
  let branches = $state<BranchInfo[]>([]);
  let selectedCommits = $state<Set<string>>(new Set());
  let loading = $state(false);
  let error = $state<string | null>(null);

  // Modal states
  const editModal = createModalState<CommitInfo>();
  const squashModal = createModalState();
  const diffModal = createAsyncModalState<CommitDiff>();

  // Form state for modals
  let editMessage = $state('');
  let squashMessage = $state('');

  // Auto-stash option
  let autoStash = $state(true);

  let commitListRef: CommitList | undefined = $state();

  async function openFolderDialog() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Git Repository'
    });

    if (selected) {
      await loadRepo(selected as string);
    }
  }

  async function loadRepo(path: string) {
    loading = true;
    error = null;

    try {
      const validation = await invoke<RepoValidation>('validate_repo', { path });

      if (!validation.valid) {
        error = validation.error ?? 'Not a valid Git repository';
        return;
      }

      if (!validation.has_commits) {
        error = validation.error ?? 'Repository has no commits yet';
        return;
      }

      repoInfo = await invoke<RepoInfo>('get_repo_info', { repoPath: path });
      branches = await invoke<BranchInfo[]>('get_branches', { repoPath: path });
      repoPath = path;
      selectedCommits = new Set();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function switchBranch(branchName: string) {
    if (!repoPath || branchName === repoInfo?.branch) return;

    loading = true;
    error = null;

    try {
      await invoke('switch_branch', {
        repoPath,
        branchName,
        autoStash
      });
      // Reload repo info and branches after switch
      repoInfo = await invoke<RepoInfo>('get_repo_info', { repoPath });
      branches = await invoke<BranchInfo[]>('get_branches', { repoPath });
      selectedCommits = new Set();
      commitListRef?.refresh();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function openEditModal(commit: CommitInfo) {
    editModal.open(commit);
    editMessage = commit.message;
  }

  function closeEditModal() {
    editModal.close();
    editMessage = '';
  }

  async function saveEditMessage() {
    if (!editModal.data || !repoPath) return;

    loading = true;
    error = null;

    try {
      await invoke('edit_commit_message', {
        repoPath,
        commitHash: editModal.data.hash,
        newMessage: editMessage,
        autoStash
      });
      closeEditModal();
      commitListRef?.refresh();
      selectedCommits = new Set();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function openSquashModal() {
    if (selectedCommits.size < 2) {
      error = 'Select at least 2 commits to squash';
      return;
    }
    squashMessage = '';
    squashModal.open();
  }

  function closeSquashModal() {
    squashModal.close();
    squashMessage = '';
  }

  async function performSquash() {
    if (!repoPath || selectedCommits.size < 2) return;

    loading = true;
    error = null;

    try {
      await invoke('squash_commits', {
        repoPath,
        commitHashes: Array.from(selectedCommits),
        newMessage: squashMessage,
        autoStash
      });
      closeSquashModal();
      commitListRef?.refresh();
      selectedCommits = new Set();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function handleSelectionChange(newSelection: Set<string>) {
    selectedCommits = newSelection;
  }

  async function openDiffModal(commit: CommitInfo) {
    if (!repoPath) return;

    diffModal.open();
    diffModal.setLoading(true);

    try {
      const data = await invoke<CommitDiff>('get_commit_diff', {
        repoPath,
        commitHash: commit.hash
      });
      diffModal.setData(data);
    } catch (e) {
      error = String(e);
      diffModal.close();
    } finally {
      diffModal.setLoading(false);
    }
  }

  function closeDiffModal() {
    diffModal.close();
  }
</script>

<main>
  <header data-tauri-drag-region>
    <div class="header-left" data-tauri-drag-region>
      <h1 data-tauri-drag-region>Git Rewrite</h1>
      {#if repoInfo}
        <div class="repo-badge">
          <span class="repo-name">{repoInfo.name}</span>
          <select
            class="branch-select"
            value={repoInfo.branch}
            onchange={(e) => switchBranch(e.currentTarget.value)}
          >
            {#each branches as branch}
              <option value={branch.name}>{branch.name}</option>
            {/each}
          </select>
        </div>
      {/if}
    </div>
    <div class="header-actions">
      <ThemeToggle />
      <button class="btn secondary" onclick={openFolderDialog}>
        Open Repository
      </button>
      {#if selectedCommits.size >= 2}
        <button class="btn primary" onclick={openSquashModal}>
          Squash {selectedCommits.size} commits
        </button>
      {/if}
    </div>
  </header>

  {#if error}
    <div class="error-banner">
      {error}
      <button class="dismiss-btn" onclick={() => error = null}>Dismiss</button>
    </div>
  {/if}

  {#if loading}
    <div class="loading-overlay">
      <div class="spinner"></div>
    </div>
  {/if}

  <div class="content">
    {#if repoPath}
      <CommitList
        bind:this={commitListRef}
        {repoPath}
        onEditCommit={openEditModal}
        onViewDiff={openDiffModal}
        {selectedCommits}
        onSelectionChange={handleSelectionChange}
      />
    {:else}
      <div class="welcome">
        <div class="welcome-icon">
          <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="12" cy="12" r="4"/>
            <line x1="1.05" y1="12" x2="7" y2="12"/>
            <line x1="17.01" y1="12" x2="22.96" y2="12"/>
          </svg>
        </div>
        <h2>Welcome to Git Rewrite</h2>
        <p>Easily edit commit messages and squash commits in your Git repositories.</p>
        <button class="btn primary large" onclick={openFolderDialog}>
          Open a Repository
        </button>
      </div>
    {/if}
  </div>
</main>

<!-- Edit Message Modal -->
<Modal open={editModal.isOpen} title="Edit Commit Message" onClose={closeEditModal}>
  {#snippet children()}
    {#if editModal.data}
      <div class="edit-form">
        <div class="commit-preview">
          <span class="hash">{editModal.data.short_hash}</span>
          <span class="author">{editModal.data.author}</span>
        </div>
        <textarea
          bind:value={editMessage}
          placeholder="Enter commit message..."
          rows="6"
        ></textarea>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={autoStash} />
          Auto-stash uncommitted changes
        </label>
      </div>
    {/if}
  {/snippet}
  {#snippet footer()}
    <button class="btn secondary" onclick={closeEditModal}>Cancel</button>
    <button class="btn primary" onclick={saveEditMessage} disabled={!editMessage.trim()}>
      Save
    </button>
  {/snippet}
</Modal>

<!-- Squash Modal -->
<Modal open={squashModal.isOpen} title="Squash Commits" onClose={closeSquashModal}>
  {#snippet children()}
    <div class="squash-form">
      <p class="squash-info">
        Squashing <strong>{selectedCommits.size}</strong> commits into one.
      </p>
      <label for="squash-message">New commit message:</label>
      <textarea
        id="squash-message"
        bind:value={squashMessage}
        placeholder="Enter message for the squashed commit..."
        rows="6"
      ></textarea>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={autoStash} />
        Auto-stash uncommitted changes
      </label>
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn secondary" onclick={closeSquashModal}>Cancel</button>
    <button class="btn primary" onclick={performSquash} disabled={!squashMessage.trim()}>
      Squash Commits
    </button>
  {/snippet}
</Modal>

<!-- Diff Modal -->
<svelte:window onkeydown={(e) => e.key === 'Escape' && diffModal.isOpen && closeDiffModal()} />
{#if diffModal.isOpen}
  <div
    class="diff-modal-backdrop"
    role="presentation"
    onclick={closeDiffModal}
    onkeydown={(e) => e.key === 'Escape' && closeDiffModal()}
  >
    <div
      class="diff-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="diff-modal-title"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <div class="diff-modal-header">
        <div class="diff-modal-title" id="diff-modal-title">
          {#if diffModal.data}
            <span class="hash">{diffModal.data.hash.slice(0, 7)}</span>
            <span class="message">{diffModal.data.message.split('\n')[0]}</span>
          {:else}
            Loading diff...
          {/if}
        </div>
        <button class="close-btn" onclick={closeDiffModal} aria-label="Close">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
      <div class="diff-modal-body">
        {#if diffModal.loading}
          <div class="diff-loading">
            <div class="spinner"></div>
            <span>Loading diff...</span>
          </div>
        {:else if diffModal.data}
          <DiffViewer diff={diffModal.data} />
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  /* Page-specific styles - Liquid Glass Apple Style */

  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: transparent;
  }

  /* Light mode: add subtle background for readability with vibrancy */
  :global([data-theme="light"]) main {
    background: rgba(255, 255, 255, 0.7);
  }

  :global([data-theme="light"]) header {
    background: rgba(255, 255, 255, 0.8) !important;
  }

  header {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 14px 24px;
    padding-left: 80px; /* Space for macOS traffic lights */
    background: var(--glass-bg);
    backdrop-filter: blur(var(--blur-xl));
    -webkit-backdrop-filter: blur(var(--blur-xl));
    border-bottom: 1px solid var(--glass-border);
    box-shadow:
      var(--shadow-inner),
      0 4px 24px rgba(0, 0, 0, 0.2);
    position: relative;
    z-index: 10;
  }

  header::before {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(
      180deg,
      rgba(255, 255, 255, 0.05) 0%,
      transparent 100%
    );
    pointer-events: none;
    border-radius: inherit;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
    min-width: 0;
    flex-shrink: 1;
  }

  .header-actions {
    display: flex;
    gap: 10px;
    flex-shrink: 0;
  }

  h1 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
    background: linear-gradient(135deg, #fff 0%, rgba(255, 255, 255, 0.7) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    letter-spacing: -0.03em;
  }

  .repo-badge {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 14px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--blur-sm));
    -webkit-backdrop-filter: blur(var(--blur-sm));
    border: 1px solid var(--glass-border-light);
    border-radius: var(--radius-lg);
    font-size: 13px;
    box-shadow: var(--shadow-inner);
    min-width: 0;
    flex-shrink: 1;
  }

  .repo-name {
    font-weight: 600;
    color: var(--text-color);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 150px;
  }

  .branch-select {
    background: rgba(0, 0, 0, 0.3);
    color: var(--accent-hover);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    padding: 6px 28px 6px 10px;
    font-size: 13px;
    font-weight: 500;
    width: auto;
    cursor: pointer;
    transition: all var(--transition-fast);
    backdrop-filter: blur(var(--blur-sm));
    -webkit-backdrop-filter: blur(var(--blur-sm));
  }

  .branch-select:hover,
  .branch-select:focus {
    border-color: var(--accent-color);
    box-shadow:
      0 0 0 3px var(--accent-muted),
      0 0 16px var(--accent-glow);
    outline: none;
  }

  .content {
    flex: 1;
    overflow: hidden;
    padding: 20px;
  }

  /* Welcome screen */
  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    padding: 40px;
  }

  .welcome-icon {
    margin-bottom: 32px;
    padding: 28px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--blur-lg));
    -webkit-backdrop-filter: blur(var(--blur-lg));
    border: 1px solid var(--glass-border-light);
    border-radius: var(--radius-xl);
    box-shadow:
      var(--shadow-inner),
      var(--shadow-glow);
  }

  .welcome-icon svg {
    color: var(--accent-color);
    filter: drop-shadow(0 0 12px var(--accent-glow));
  }

  .welcome h2 {
    margin: 0 0 16px;
    font-size: 32px;
    font-weight: 700;
    background: linear-gradient(135deg, #fff 0%, rgba(255, 255, 255, 0.6) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    letter-spacing: -0.03em;
  }

  .welcome p {
    margin: 0 0 40px;
    max-width: 440px;
    line-height: 1.7;
    color: var(--text-secondary);
    font-size: 15px;
  }

  /* Modal forms */
  .edit-form,
  .squash-form {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .commit-preview {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 16px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--blur-sm));
    -webkit-backdrop-filter: blur(var(--blur-sm));
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    font-size: 13px;
    box-shadow: var(--shadow-inner);
  }

  .commit-preview .author {
    color: var(--text-secondary);
  }

  .squash-info {
    margin: 0;
    padding: 14px 16px;
    background: linear-gradient(135deg, var(--accent-muted) 0%, rgba(139, 92, 246, 0.1) 100%);
    border: 1px solid rgba(99, 102, 241, 0.2);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: 14px;
  }

  .squash-info strong {
    color: var(--accent-hover);
    font-weight: 600;
  }

  /* Diff Modal - larger for viewing diffs */
  .diff-modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: fadeIn 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .diff-modal {
    background: rgba(30, 30, 45, 0.85);
    backdrop-filter: blur(40px) saturate(180%);
    -webkit-backdrop-filter: blur(40px) saturate(180%);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-xl);
    box-shadow:
      0 25px 50px -12px rgba(0, 0, 0, 0.5),
      0 0 0 1px rgba(255, 255, 255, 0.05) inset,
      0 1px 0 rgba(255, 255, 255, 0.1) inset;
    width: 90%;
    max-width: 1000px;
    height: 85vh;
    display: flex;
    flex-direction: column;
    animation: slideIn 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;
  }

  /* Light mode diff modal */
  :global([data-theme="light"]) .diff-modal {
    background: rgba(255, 255, 255, 0.85);
    border: 1px solid rgba(0, 0, 0, 0.1);
    box-shadow:
      0 25px 50px -12px rgba(0, 0, 0, 0.25),
      0 0 0 1px rgba(0, 0, 0, 0.05) inset;
  }

  :global([data-theme="light"]) .diff-modal-backdrop {
    background: rgba(255, 255, 255, 0.3);
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: scale(0.95) translateY(-20px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .diff-modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 18px 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    flex-shrink: 0;
    background: rgba(255, 255, 255, 0.03);
  }

  :global([data-theme="light"]) .diff-modal-header {
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
    background: rgba(0, 0, 0, 0.02);
  }

  .diff-modal-title {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
  }

  .diff-modal-title .message {
    font-weight: 500;
    color: var(--text-color);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .diff-modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    background: transparent;
  }

  .diff-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    height: 200px;
    color: var(--muted-color);
  }

  .close-btn {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    cursor: pointer;
    color: var(--muted-color);
    padding: 8px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.15);
    color: var(--text-color);
    border-color: rgba(255, 255, 255, 0.2);
  }

  :global([data-theme="light"]) .close-btn {
    background: rgba(0, 0, 0, 0.05);
    border: 1px solid rgba(0, 0, 0, 0.1);
  }

  :global([data-theme="light"]) .close-btn:hover {
    background: rgba(0, 0, 0, 0.1);
    border-color: rgba(0, 0, 0, 0.15);
  }

  /* ============================================
     Responsive Styles
     ============================================ */

  /* Medium screens (< 900px) - compact header */
  @media (max-width: 900px) {
    .repo-name {
      max-width: 100px;
    }

    .header-actions .btn {
      padding: 8px 14px;
      font-size: 13px;
    }
  }

  /* Tablet (< 768px) */
  @media (max-width: 768px) {
    header {
      flex-direction: column;
      gap: 12px;
      padding: 12px 16px;
    }

    .header-left {
      width: 100%;
      justify-content: space-between;
      gap: 12px;
    }

    .repo-name {
      max-width: 120px;
    }

    .header-actions {
      width: 100%;
      justify-content: stretch;
    }

    .header-actions .btn {
      flex: 1;
      padding: 10px 16px;
      font-size: 14px;
    }

    .content {
      padding: 12px;
    }

    .welcome {
      padding: 24px 16px;
    }

    .welcome-icon {
      padding: 20px;
      margin-bottom: 24px;
    }

    .welcome-icon svg {
      width: 48px;
      height: 48px;
    }

    .welcome h2 {
      font-size: 24px;
    }

    .welcome p {
      font-size: 14px;
      margin-bottom: 28px;
    }

    /* Diff Modal */
    .diff-modal {
      width: 100%;
      height: 100vh;
      max-width: none;
      border-radius: 0;
    }

    .diff-modal-header {
      padding: 14px 16px;
    }

    .diff-modal-body {
      padding: 16px;
    }
  }

  /* Mobile (< 480px) */
  @media (max-width: 480px) {
    header {
      padding: 10px 12px;
    }

    h1 {
      font-size: 17px;
    }

    .repo-badge {
      padding: 6px 10px;
      gap: 8px;
      font-size: 12px;
    }

    .repo-name {
      max-width: 100px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .branch-select {
      font-size: 12px;
      padding: 5px 24px 5px 8px;
    }

    .header-actions {
      flex-direction: column;
      gap: 8px;
    }

    .content {
      padding: 8px;
    }

    .welcome-icon {
      padding: 16px;
    }

    .welcome-icon svg {
      width: 40px;
      height: 40px;
    }

    .welcome h2 {
      font-size: 20px;
    }

    .welcome p {
      font-size: 13px;
    }

    .edit-form,
    .squash-form {
      gap: 14px;
    }

    .commit-preview {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .diff-modal-header {
      padding: 12px;
    }

    .diff-modal-title {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .diff-modal-title .message {
      white-space: normal;
      font-size: 13px;
    }

    .diff-modal-body {
      padding: 12px;
    }
  }
</style>
