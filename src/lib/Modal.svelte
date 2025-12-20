<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    open: boolean;
    title: string;
    onClose: () => void;
    children: Snippet;
    footer?: Snippet;
  }

  let { open, title, onClose, children, footer }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div class="modal-backdrop" onclick={handleBackdropClick}>
    <div class="modal">
      <div class="modal-header">
        <h2>{title}</h2>
        <button class="close-btn" onclick={onClose}>
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
      <div class="modal-body">
        {@render children()}
      </div>
      {#if footer}
        <div class="modal-footer">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(var(--blur-lg));
    -webkit-backdrop-filter: blur(var(--blur-lg));
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

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: scale(0.92) translateY(-24px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .modal {
    background: var(--modal-bg);
    backdrop-filter: blur(var(--blur-xl));
    -webkit-backdrop-filter: blur(var(--blur-xl));
    border: 1px solid var(--glass-border-light);
    border-radius: var(--radius-xl);
    box-shadow:
      var(--shadow-lg),
      var(--shadow-glow),
      inset 0 1px 0 rgba(255, 255, 255, 0.06);
    max-width: 560px;
    width: 90%;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    animation: slideIn 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 18px 24px;
    border-bottom: 1px solid var(--glass-border);
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.04) 0%, transparent 100%);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 17px;
    font-weight: 600;
    background: linear-gradient(135deg, #fff 0%, rgba(255, 255, 255, 0.75) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .close-btn {
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
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
    background: var(--glass-bg-hover);
    color: var(--text-color);
    border-color: var(--glass-border-light);
    transform: scale(1.05);
  }

  .modal-body {
    padding: 24px;
    overflow-y: auto;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 18px 24px;
    border-top: 1px solid var(--glass-border);
    background: linear-gradient(180deg, transparent 0%, rgba(0, 0, 0, 0.2) 100%);
    border-radius: 0 0 var(--radius-xl) var(--radius-xl);
  }

  /* ============================================
     Responsive Styles
     ============================================ */

  /* Tablet (< 768px) */
  @media (max-width: 768px) {
    .modal {
      width: 95%;
      max-height: 90vh;
    }

    .modal-header {
      padding: 14px 18px;
    }

    .modal-body {
      padding: 18px;
    }

    .modal-footer {
      padding: 14px 18px;
      gap: 10px;
    }
  }

  /* Mobile (< 480px) */
  @media (max-width: 480px) {
    .modal {
      width: 100%;
      max-width: none;
      max-height: 100vh;
      height: 100vh;
      border-radius: 0;
    }

    .modal-header {
      padding: 12px 16px;
    }

    .modal-header h2 {
      font-size: 15px;
    }

    .modal-body {
      padding: 16px;
      flex: 1;
    }

    .modal-footer {
      padding: 12px 16px;
      flex-direction: column;
      border-radius: 0;
    }

    .modal-footer :global(.btn) {
      width: 100%;
    }
  }
</style>
