<script lang="ts">
  import { Button, Card, CardHeader, SectionLabel } from "@liberte-top/components";
  import InfoChip from "./lib/components/InfoChip.svelte";
  import { config } from "./lib/config";
  import MetricTile from "./lib/components/MetricTile.svelte";
  import StatusPill from "./lib/components/StatusPill.svelte";
  import { onMount } from "svelte";

  const envLabel = config.envLabel;

  type Viewer = {
    subject: string | null;
    auth_type: string | null;
    scopes: string[];
  };

  type NotesResponse = {
    viewer: Viewer;
    notes: Array<{ id: string; title: string; summary: string }>;
  };

  type AuthState = {
    ready: boolean;
    authenticated: boolean;
    subject: string | null;
    authType: string | null;
    scopes: string[];
  };

  let healthText = "loading";
  let viewer: Viewer | null = null;
  let notes: NotesResponse | null = null;
  let errorText = "";
  let authState: AuthState = {
    ready: false,
    authenticated: false,
    subject: null,
    authType: null,
    scopes: [],
  };
  let writeResult = "not attempted";
  let dataStatus = "checking";
  let lastRefresh = "never";

  $: viewerLabel = viewer?.subject ? `${viewer.subject.slice(0, 8)}...` : "anonymous";

  function healthTone() {
    return healthText === "ok" ? "success" : "danger";
  }

  function authTone() {
    return authState.authenticated ? "ready" : "idle";
  }

  function hasAnyScope(required: string[]) {
    return required.some((scope) => authState.scopes.includes(scope));
  }

  function hasAllScopes(required: string[]) {
    return required.every((scope) => authState.scopes.includes(scope));
  }

  function syncAuthState(nextViewer: Viewer | null) {
    authState = {
      ready: true,
      authenticated: Boolean(nextViewer?.subject),
      subject: nextViewer?.subject ?? null,
      authType: nextViewer?.auth_type ?? null,
      scopes: nextViewer?.scopes ?? [],
    };
  }

  async function refreshDashboard(showResult = false) {
    dataStatus = "checking";
    errorText = "";
    try {
      const [healthRes, viewerRes, notesRes] = await Promise.all([
        fetch("/api/v1/health"),
        fetch("/api/v1/viewer"),
        fetch("/api/v1/notes"),
      ]);

      if (!healthRes.ok) {
        throw new Error(`health failed: ${healthRes.status}`);
      }

      if (!viewerRes.ok) {
        throw new Error(`viewer failed: ${viewerRes.status}`);
      }

      if (!notesRes.ok) {
        throw new Error(`notes failed: ${notesRes.status}`);
      }

      const [health, nextViewer, nextNotes] = await Promise.all([
        healthRes.json(),
        viewerRes.json(),
        notesRes.json(),
      ]);

      healthText = health.status;
      viewer = nextViewer;
      notes = nextNotes;
      syncAuthState(viewer);
      dataStatus = "ready";
      lastRefresh = new Date().toLocaleTimeString();
      if (showResult) {
        writeResult = "dashboard refreshed";
      }
    } catch (error) {
      healthText = "unreachable";
      errorText = error instanceof Error ? error.message : "request failed";
      syncAuthState(null);
      dataStatus = "error";
      lastRefresh = new Date().toLocaleTimeString();
    }
  }

  onMount(async () => {
    await refreshDashboard();
  });

  async function tryWrite() {
    try {
      const response = await fetch("/api/v1/notes", {
        method: "POST",
        headers: {
          "content-type": "application/json",
        },
        body: JSON.stringify({ title: "blocked-by-scope" }),
      });

      if (response.ok) {
        writeResult = "unexpected success";
        return;
      }

      writeResult = `status ${response.status}`;
    } catch (error) {
      writeResult = error instanceof Error ? error.message : "unknown";
    }
  }
</script>

<main class="shell">
  <Card class="hero-card">
    <div>
      <SectionLabel>app.smoke</SectionLabel>
      <h1>Gateway-authenticated business app sample</h1>
      <p class="lede">This page stays business-focused: identity comes from upstream auth, and the UI only reflects the trusted viewer context.</p>
    </div>

    <div class="hero-metrics">
      <MetricTile label="Environment" value={envLabel} />
      <MetricTile label="Health" tone={healthTone()}>
        <strong class={healthTone()}>{healthText}</strong>
      </MetricTile>
      <MetricTile label="Viewer" value={viewerLabel} />
      <MetricTile label="Refresh" value={lastRefresh} />
    </div>
  </Card>

  <section class="dashboard">
    <Card class="identity-card">
      <div class="section-head">
        <CardHeader>
          <SectionLabel>Identity</SectionLabel>
          <h2>Gateway-provided viewer</h2>
        </CardHeader>
        <StatusPill tone={authTone()}>{authState.authenticated ? "authenticated" : "anonymous"}</StatusPill>
      </div>

      {#if viewer}
        <dl class="identity-grid">
          <div>
            <dt>Subject</dt>
            <dd><code>{viewer.subject}</code></dd>
          </div>
          <div>
            <dt>Auth type</dt>
            <dd>{viewer.auth_type}</dd>
          </div>
          <div>
            <dt>Scopes</dt>
            <dd>{viewer.scopes.join(", ") || "none"}</dd>
          </div>
        </dl>

        <div class="scope-demo">
          <InfoChip tone={hasAnyScope(["notes:read"]) ? "success" : "default"}>notes:read</InfoChip>
          <InfoChip tone={hasAllScopes(["notes:read", "profile:read"]) ? "success" : "default"}>notes:read + profile:read</InfoChip>
          <Button size="lg" variant="secondary" disabled={!hasAllScopes(["notes:write"])} onclick={tryWrite}>write action</Button>
          <Button size="lg" onclick={() => refreshDashboard(true)}>refresh dashboard</Button>
        </div>

        <p class="caption">Status: <strong>{dataStatus}</strong> · Last refresh: <strong>{lastRefresh}</strong></p>
        <p class="caption">Write smoke result: <strong>{writeResult}</strong></p>
        <pre>{JSON.stringify(authState, null, 2)}</pre>
      {:else}
        <p>{errorText || "Loading viewer context..."}</p>
        <div class="scope-demo">
          <Button size="lg" onclick={() => refreshDashboard(true)}>refresh dashboard</Button>
        </div>
      {/if}
    </Card>

    <Card class="notes-card">
      <div class="section-head">
        <CardHeader>
          <SectionLabel>Payload</SectionLabel>
          <h2>Sample notes</h2>
        </CardHeader>
      </div>

      {#if notes}
        <div class="note-list">
          {#each notes.notes as note}
            <article class="note-item">
              <p class="note-id">{note.id}</p>
              <h3>{note.title}</h3>
              <p>{note.summary}</p>
            </article>
          {/each}
        </div>
        <pre>{JSON.stringify(notes, null, 2)}</pre>
      {:else}
        <p>{errorText || "Loading notes..."}</p>
      {/if}
    </Card>
  </section>
</main>

<style>
  :global(:root) {
    --lt-page-bg: #eef3f7;
    --lt-color-primary: #6f5a2f;
    --lt-color-primary-hover: #594822;
    --lt-color-on-primary: #fffdf8;
    --lt-color-surface: rgba(255, 255, 255, 0.86);
    --lt-color-surface-hover: #f4efe6;
    --lt-color-border: rgba(29, 39, 51, 0.12);
    --lt-color-border-muted: rgba(29, 39, 51, 0.08);
    --lt-color-text: #1d2733;
    --lt-color-text-muted: #5d6774;
    --lt-color-text-subtle: #6c7784;
    --lt-color-fill-muted: #f4efe6;
    --lt-color-success: #215f35;
    --lt-color-success-surface: #dff2e4;
    --lt-color-success-border: rgba(33, 95, 53, 0.2);
    --lt-color-danger: #8a3f2f;
    --lt-color-danger-surface: #f6e4de;
    --lt-color-danger-border: rgba(138, 63, 47, 0.2);
    --lt-card-radius: 1rem;
    --lt-card-padding: 1.25rem;
    --lt-card-shadow: 0 12px 30px rgba(29, 39, 51, 0.08);
    --lt-button-radius: 999px;
    --lt-button-padding-inline: 0.9rem;
    --lt-button-height-lg: 2.4rem;
    --lt-button-font-size-lg: 0.92rem;
  }

  :global(body) {
    background:
      radial-gradient(circle at top left, rgba(213, 225, 235, 0.7), transparent 32%),
      linear-gradient(180deg, #f4efe6 0%, #eef3f7 100%);
  }

  .shell {
    min-height: 100vh;
    padding: 2rem;
    display: grid;
    gap: 1rem;
  }

  .dashboard {
    display: grid;
    grid-template-columns: minmax(320px, 0.9fr) minmax(0, 1.1fr);
    gap: 1rem;
  }

  :global(.hero-card) {
    display: grid;
    grid-template-columns: minmax(0, 1.3fr) minmax(240px, 0.7fr);
    gap: 1rem;
    align-items: start;
  }

  .lede {
    margin: 0;
    color: #5d6774;
    max-width: 54rem;
  }

  .hero-metrics {
    display: grid;
    gap: 0.75rem;
  }

  dt,
  .note-id,
  .caption {
    display: block;
    font-size: 0.8rem;
    color: #6c7784;
  }

  .success {
    color: var(--lt-color-success);
  }

  .danger {
    color: var(--lt-color-danger);
  }

  .section-head {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: start;
    margin-bottom: 1rem;
  }

  .section-head :global(.lt-card-header) {
    margin-bottom: 0;
  }

  .section-head :global(.lt-section-label) {
    color: #7b5d2b;
    letter-spacing: 0.08em;
    margin-bottom: 0.45rem;
  }

  h1, h2 {
    margin: 0 0 0.75rem;
  }

  pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-size: 0.92rem;
  }

  code {
    font-family: "IBM Plex Mono", monospace;
  }

  .identity-grid {
    display: grid;
    gap: 0.95rem;
    margin: 0 0 1rem;
  }

  .identity-grid dd {
    margin: 0.2rem 0 0;
  }

  .scope-demo {
    margin-top: 1rem;
    display: flex;
    gap: 0.75rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .note-list {
    display: grid;
    gap: 0.8rem;
    margin-bottom: 1rem;
  }

  .note-item {
    padding: 1rem;
    border-radius: 0.9rem;
    background: #f7f3ea;
    border: 1px solid rgba(29, 39, 51, 0.08);
  }

  .note-item h3,
  .note-item p {
    margin: 0;
  }

  .note-item h3 {
    margin-top: 0.25rem;
    margin-bottom: 0.35rem;
  }

  @media (max-width: 780px) {
    .hero,
    .dashboard {
      grid-template-columns: 1fr;
    }
  }
</style>
