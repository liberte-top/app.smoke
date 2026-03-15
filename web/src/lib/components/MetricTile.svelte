<script lang="ts">
  type Props = {
    label: string;
    value?: string;
    tone?: "default" | "success" | "danger";
    children?: import("svelte").Snippet;
  };

  let {
    label,
    value,
    tone = "default",
    children,
  }: Props = $props();
</script>

<div class={`metric-tile metric-tile--${tone}`}>
  <span class="metric-tile__label">{label}</span>
  {#if children}
    {@render children()}
  {:else}
    <strong>{value}</strong>
  {/if}
</div>

<style>
  .metric-tile {
    padding: 0.95rem 1rem;
    border-radius: 0.9rem;
    background: var(--smoke-metric-bg, rgba(243, 238, 229, 0.92));
    border: 1px solid var(--smoke-metric-border, var(--lt-color-border-muted));
  }

  .metric-tile__label {
    display: block;
    font-size: 0.8rem;
    color: var(--lt-color-text-subtle);
  }

  .metric-tile--success :global(strong) {
    color: var(--lt-color-success);
  }

  .metric-tile--danger :global(strong) {
    color: var(--lt-color-danger);
  }
</style>
