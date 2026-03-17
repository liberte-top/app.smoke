<script lang="ts">
  type Props = {
    tone?: "idle" | "ready" | "danger" | "loading";
    children?: import("svelte").Snippet;
  };

  let {
    tone = "idle",
    children,
  }: Props = $props();
</script>

<span class={`status-pill status-pill--${tone}`}>
  {@render children?.()}
</span>

<style>
  .status-pill {
    border-radius: 999px;
    padding: 0.35rem 0.7rem;
    background: var(--smoke-pill-bg, #f1ece4);
    color: var(--lt-color-text-subtle);
    border: 1px solid var(--lt-color-border);
    font-size: 0.82rem;
  }

  .status-pill--ready {
    background: var(--lt-color-success-surface);
    color: var(--lt-color-success);
    border-color: var(--lt-color-success-border);
  }

  .status-pill--danger {
    background: var(--lt-color-danger-surface);
    color: var(--lt-color-danger);
    border-color: var(--lt-color-danger-border);
  }

  .status-pill--loading {
    background: rgba(111, 90, 47, 0.08);
    color: #6f5a2f;
  }
</style>
