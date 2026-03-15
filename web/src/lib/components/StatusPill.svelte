<script lang="ts">
  type Props = {
    tone?: "idle" | "ready";
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
</style>
