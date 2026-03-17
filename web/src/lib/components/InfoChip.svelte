<script lang="ts">
  type Props = {
    tone?: "default" | "success" | "danger";
    children?: import("svelte").Snippet;
  };

  let {
    tone = "default",
    children,
  }: Props = $props();
</script>

<span class={`info-chip info-chip--${tone}`}>
  {@render children?.()}
</span>

<style>
  .info-chip {
    border-radius: 999px;
    border: 1px solid rgba(29, 39, 51, 0.14);
    padding: 0.35rem 0.7rem;
    background: var(--lt-color-fill-muted);
    color: var(--lt-color-text-subtle);
  }

  .info-chip--success {
    background: var(--lt-color-success-surface);
    color: var(--lt-color-success);
    border-color: var(--lt-color-success-border);
  }

  .info-chip--danger {
    background: var(--lt-color-danger-surface);
    color: var(--lt-color-danger);
    border-color: var(--lt-color-danger-border);
  }
</style>
