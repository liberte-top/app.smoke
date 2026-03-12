<script lang="ts">
  import { onMount } from "svelte";
  import axios from "axios";

  const envLabel = import.meta.env.VITE_ENV_LABEL ?? "local";
  const http = axios.create();

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
    return healthText === "ok" ? "healthy" : "degraded";
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
      const [health, viewerRes, notesRes] = await Promise.all([
        http.get("/api/v1/health"),
        http.get("/api/v1/viewer"),
        http.get("/api/v1/notes"),
      ]);

      healthText = health.data.status;
      viewer = viewerRes.data;
      notes = notesRes.data;
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
      await http.post("/api/v1/notes", { title: "blocked-by-scope" });
      writeResult = "unexpected success";
    } catch (error: any) {
      writeResult = `status ${error?.response?.status ?? "unknown"}`;
    }
  }
</script>

<main class="shell">
  <section class="hero card">
    <div>
      <p class="eyebrow">app.smoke</p>
      <h1>Gateway-authenticated business app sample</h1>
      <p class="lede">This page stays business-focused: identity comes from upstream auth, and the UI only reflects the trusted viewer context.</p>
    </div>

    <div class="hero-metrics">
      <div>
        <span class="metric-label">Environment</span>
        <strong>{envLabel}</strong>
      </div>
      <div>
        <span class="metric-label">Health</span>
        <strong class={healthTone()}>{healthText}</strong>
      </div>
      <div>
        <span class="metric-label">Viewer</span>
        <strong>{viewerLabel}</strong>
      </div>
      <div>
        <span class="metric-label">Refresh</span>
        <strong>{lastRefresh}</strong>
      </div>
    </div>
  </section>

  <section class="dashboard">
    <section class="card identity-card">
      <div class="section-head">
        <div>
          <p class="eyebrow">Identity</p>
          <h2>Gateway-provided viewer</h2>
        </div>
        <span class:ready={authState.authenticated} class="pill">{authState.authenticated ? "authenticated" : "anonymous"}</span>
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
          <span class:ok={hasAnyScope(["notes:read"])}>notes:read</span>
          <span class:ok={hasAllScopes(["notes:read", "profile:read"])}>notes:read + profile:read</span>
          <button disabled={!hasAllScopes(["notes:write"])} on:click={tryWrite}>write action</button>
          <button on:click={() => refreshDashboard(true)}>refresh dashboard</button>
        </div>

        <p class="caption">Status: <strong>{dataStatus}</strong> · Last refresh: <strong>{lastRefresh}</strong></p>
        <p class="caption">Write smoke result: <strong>{writeResult}</strong></p>
        <pre>{JSON.stringify(authState, null, 2)}</pre>
      {:else}
        <p>{errorText || "Loading viewer context..."}</p>
        <div class="scope-demo">
          <button on:click={() => refreshDashboard(true)}>refresh dashboard</button>
        </div>
      {/if}
    </section>

    <section class="card notes-card">
      <div class="section-head">
        <div>
          <p class="eyebrow">Payload</p>
          <h2>Sample notes</h2>
        </div>
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
    </section>
  </section>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
    background:
      radial-gradient(circle at top left, rgba(213, 225, 235, 0.7), transparent 32%),
      linear-gradient(180deg, #f4efe6 0%, #eef3f7 100%);
    color: #1d2733;
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

  .card {
    background: rgba(255, 255, 255, 0.86);
    border: 1px solid rgba(29, 39, 51, 0.12);
    border-radius: 1rem;
    padding: 1.25rem;
    box-shadow: 0 12px 30px rgba(29, 39, 51, 0.08);
  }

  .hero {
    display: grid;
    grid-template-columns: minmax(0, 1.3fr) minmax(240px, 0.7fr);
    gap: 1rem;
    align-items: start;
  }

  .eyebrow {
    margin: 0 0 0.45rem;
    color: #7b5d2b;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-size: 0.75rem;
    font-weight: 700;
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

  .hero-metrics > div {
    padding: 0.95rem 1rem;
    border-radius: 0.9rem;
    background: rgba(243, 238, 229, 0.92);
    border: 1px solid rgba(29, 39, 51, 0.08);
  }

  .metric-label,
  dt,
  .note-id,
  .caption {
    display: block;
    font-size: 0.8rem;
    color: #6c7784;
  }

  .healthy {
    color: #215f35;
  }

  .degraded {
    color: #8a3f2f;
  }

  .section-head {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: start;
    margin-bottom: 1rem;
  }

  .pill {
    border-radius: 999px;
    padding: 0.35rem 0.7rem;
    background: #f1ece4;
    color: #6c7784;
    border: 1px solid rgba(29, 39, 51, 0.1);
    font-size: 0.82rem;
  }

  .pill.ready {
    background: #dff2e4;
    color: #215f35;
    border-color: rgba(33, 95, 53, 0.2);
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

  .scope-demo span,
  .scope-demo button {
    border-radius: 999px;
    border: 1px solid rgba(29, 39, 51, 0.14);
    padding: 0.35rem 0.7rem;
    background: #f4efe6;
    color: #6c7784;
  }

  .scope-demo .ok {
    background: #dff2e4;
    color: #215f35;
    border-color: rgba(33, 95, 53, 0.2);
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
