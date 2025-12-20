<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createGitgraph, templateExtend, TemplateName } from '@gitgraph/js';
  import type { CommitInfo } from './types';

  interface Props {
    commits: CommitInfo[];
    selectedCommits: Set<string>;
    onCommitClick?: (hash: string) => void;
  }

  let { commits, selectedCommits, onCommitClick }: Props = $props();

  let containerEl: HTMLDivElement;
  let gitgraph: ReturnType<typeof createGitgraph> | null = null;

  // Color palette for branches
  const branchColors = [
    '#6366f1', // indigo
    '#8b5cf6', // violet
    '#ec4899', // pink
    '#f97316', // orange
    '#22c55e', // green
    '#06b6d4', // cyan
    '#3b82f6', // blue
    '#eab308', // yellow
  ];

  function buildGraph() {
    if (!containerEl || commits.length === 0) return;

    // Clear previous graph
    containerEl.innerHTML = '';

    // Custom template for dark theme
    const template = templateExtend(TemplateName.Metro, {
      colors: branchColors,
      branch: {
        lineWidth: 2,
        spacing: 30,
        label: {
          display: false,
        },
      },
      commit: {
        spacing: 50,
        dot: {
          size: 8,
          strokeWidth: 2,
        },
        message: {
          display: false, // We show our own commit list
        },
      },
    });

    gitgraph = createGitgraph(containerEl, {
      template,
    });

    // Build commit index for quick lookup
    const commitMap = new Map<string, CommitInfo>();
    commits.forEach(c => commitMap.set(c.hash, c));

    // Track branches: hash -> branch object
    const branches = new Map<string, ReturnType<typeof gitgraph.branch>>();

    // Process commits from oldest to newest
    const sortedCommits = [...commits].reverse();

    // Track which commits have been processed
    const processed = new Set<string>();

    // First pass: identify branch structure
    // A commit creates a "branch point" if it has multiple children (multiple commits have it as parent)
    const childCount = new Map<string, number>();
    sortedCommits.forEach(commit => {
      commit.parent_ids.forEach(parentId => {
        childCount.set(parentId, (childCount.get(parentId) || 0) + 1);
      });
    });

    // Main branch starts from first commit
    let mainBranch = gitgraph.branch({
      name: 'main',
      style: { color: branchColors[0] },
    });
    let currentBranch = mainBranch;
    let branchCounter = 1;

    // Process each commit
    for (const commit of sortedCommits) {
      const isMerge = commit.parent_ids.length > 1;
      const isSelected = selectedCommits.has(commit.hash);

      if (isMerge) {
        // Merge commit - merge into current branch
        // For simplicity, just show it as a commit with a note
        currentBranch.commit({
          subject: commit.short_hash,
          hash: commit.hash,
          style: {
            dot: {
              color: isSelected ? '#ffffff' : undefined,
              strokeColor: isSelected ? branchColors[0] : undefined,
              strokeWidth: isSelected ? 3 : 2,
            },
          },
          onMessageClick: () => onCommitClick?.(commit.hash),
        });
      } else {
        // Regular commit
        currentBranch.commit({
          subject: commit.short_hash,
          hash: commit.hash,
          style: {
            dot: {
              color: isSelected ? '#ffffff' : undefined,
              strokeColor: isSelected ? branchColors[branchCounter % branchColors.length] : undefined,
              strokeWidth: isSelected ? 3 : 2,
            },
          },
          onMessageClick: () => onCommitClick?.(commit.hash),
        });
      }

      // If this commit is a branch point (has multiple children), create new branch for next commits
      if ((childCount.get(commit.hash) || 0) > 1) {
        branchCounter++;
        const newBranch = currentBranch.branch({
          name: `branch-${branchCounter}`,
          style: { color: branchColors[branchCounter % branchColors.length] },
        });
        // Keep on main for now
      }

      processed.add(commit.hash);
    }
  }

  onMount(() => {
    buildGraph();
  });

  // Rebuild when commits change
  $effect(() => {
    if (commits && containerEl) {
      buildGraph();
    }
  });

  onDestroy(() => {
    if (containerEl) {
      containerEl.innerHTML = '';
    }
  });
</script>

<div class="gitgraph-container" bind:this={containerEl}></div>

<style>
  .gitgraph-container {
    width: 100%;
    min-height: 200px;
    overflow: auto;
    background: var(--glass-bg);
    border-radius: var(--radius-lg);
    padding: 16px;
  }

  .gitgraph-container :global(svg) {
    display: block;
  }

  .gitgraph-container :global(.gitgraph-slot-info) {
    display: none;
  }
</style>
