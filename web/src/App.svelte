<script lang="ts">
  import { Button, Card, CardHeader, SectionLabel } from "@liberte-top/components";
  import InfoChip from "./lib/components/InfoChip.svelte";
  import { config } from "./lib/config";
  import MetricTile from "./lib/components/MetricTile.svelte";
  import StatusPill from "./lib/components/StatusPill.svelte";
  import { onMount } from "svelte";

  const envLabel = config.envLabel;
  const notesReadScopes = ["notes:read"];
  const notesWriteScopes = ["notes:write"];

  type Viewer = {
    subject: string | null;
    auth_type: string | null;
    scopes: string[];
  };

  type NotesResponse = {
    viewer: Viewer;
    notes: Array<{ id: string; title: string; summary: string }>;
  };

  type WriteResponse = {
    accepted: boolean;
    persisted: boolean;
    title: string;
    viewer: Viewer;
    message: string;
  };

  type AuthState = {
    ready: boolean;
    authenticated: boolean;
    subject: string | null;
    authType: string | null;
    scopes: string[];
  };

  type ProbeState = "idle" | "loading" | "success" | "denied" | "error";
  type ChipTone = "default" | "success" | "danger";
  type StatusTone = "idle" | "ready" | "danger" | "loading";

  type RequestProbe = {
    label: string;
    method: string;
    path: string;
    state: ProbeState;
    status: number | null;
    detail: string;
    payload: string;
    expectedScopes: string[];
  };

  const idleProbe = (label: string, method: string, path: string, expectedScopes: string[] = []): RequestProbe => ({
    label,
    method,
    path,
    state: "idle",
    status: null,
    detail: "Not attempted yet.",
    payload: "",
    expectedScopes,
  });

  let healthText = "loading";
  let healthDetail = "Waiting for API health.";
  let authState: AuthState = {
    ready: false,
    authenticated: false,
    subject: null,
    authType: null,
    scopes: [],
  };
  let viewer: Viewer | null = null;
  let notes: NotesResponse | null = null;
  let writeResponse: WriteResponse | null = null;
  let lastRefresh = "never";
  let viewerProbe = idleProbe("Viewer", "GET", "/api/v1/viewer");
  let notesProbe = idleProbe("Notes read", "GET", "/api/v1/notes", notesReadScopes);
  let writeProbe = idleProbe("Notes write", "POST", "/api/v1/notes", notesWriteScopes);

  $: viewerLabel = authState.subject ? `${authState.subject.slice(0, 12)}...` : "anonymous";
  $: authModeLabel = authState.authType ?? "none";
  $: authModeTone = chipToneForAuthMode(authState.authType);
  $: notesVisibility = notesProbe.state === "success" ? "visible" : notesProbe.state === "denied" ? "blocked" : "unknown";

  function metricTone(text: string) {
    return text === "ok" ? "success" : text === "loading" ? "default" : "danger";
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

  function hasAllScopes(required: string[]) {
    return required.every((scope) => authState.scopes.includes(scope));
  }

  function chipToneForAuthMode(authType: string | null): ChipTone {
    if (authType === "session") {
      return "success";
    }

    if (authType) {
      return "default";
    }

    return "danger";
  }

  function probeTone(state: ProbeState): StatusTone {
    if (state === "success") {
      return "ready";
    }

    if (state === "denied" || state === "error") {
      return "danger";
    }

    if (state === "loading") {
      return "loading";
    }

    return "idle";
  }

  function probeBadgeClass(state: ProbeState) {
    if (state === "success") return "probe-badge probe-badge--success";
    if (state === "denied") return "probe-badge probe-badge--denied";
    if (state === "error") return "probe-badge probe-badge--error";
    if (state === "loading") return "probe-badge probe-badge--loading";
    return "probe-badge";
  }

  function probeLabel(state: ProbeState) {
    if (state === "success") return "allowed";
    if (state === "denied") return "denied";
    if (state === "error") return "error";
    if (state === "loading") return "loading";
    return "idle";
  }

  function statusLabel(status: number | null) {
    return status === null ? "-" : `${status}`;
  }

  async function readPayload(response: Response) {
    const text = await response.text();
    if (!text) {
      return "";
    }

    try {
      return JSON.stringify(JSON.parse(text), null, 2);
    } catch {
      return text;
    }
  }

  async function runProbe(
    base: RequestProbe,
    input: RequestInfo | URL,
    init?: RequestInit,
  ) {
    try {
      const response = await fetch(input, init);
      const payload = await readPayload(response);

      if (response.ok) {
        return {
          ...base,
          state: "success" as const,
          status: response.status,
          detail: "Gateway allowed request through to app.smoke.",
          payload,
        };
      }

      if (response.status === 401 || response.status === 403) {
        return {
          ...base,
          state: "denied" as const,
          status: response.status,
          detail: "Gateway denied this request before the business app could complete it.",
          payload,
        };
      }

      return {
        ...base,
        state: "error" as const,
        status: response.status,
        detail: "Request reached the stack but did not succeed.",
        payload,
      };
    } catch (error) {
      return {
        ...base,
        state: "error" as const,
        status: null,
        detail: error instanceof Error ? error.message : "Unknown request failure.",
        payload: "",
      };
    }
  }

  async function refreshDashboard() {
    viewerProbe = { ...viewerProbe, state: "loading", detail: "Request in flight.", payload: "", status: null };
    notesProbe = { ...notesProbe, state: "loading", detail: "Request in flight.", payload: "", status: null };
    healthText = "loading";
    healthDetail = "Checking API health.";

    const [healthResult, nextViewerProbe, nextNotesProbe] = await Promise.all([
      fetch("/api/v1/health"),
      runProbe(viewerProbe, "/api/v1/viewer"),
      runProbe(notesProbe, "/api/v1/notes"),
    ]);

    viewerProbe = nextViewerProbe;
    notesProbe = nextNotesProbe;
    notes = null;

    try {
      if (!healthResult.ok) {
        throw new Error(`health failed: ${healthResult.status}`);
      }

      const health = await healthResult.json();
      healthText = health.status;
      healthDetail = "API responds on the business side.";
    } catch (error) {
      healthText = "unreachable";
      healthDetail = error instanceof Error ? error.message : "Health request failed.";
    }

    if (viewerProbe.state === "success" && viewerProbe.payload) {
      viewer = JSON.parse(viewerProbe.payload) as Viewer;
    } else {
      viewer = null;
    }

    if (notesProbe.state === "success" && notesProbe.payload) {
      notes = JSON.parse(notesProbe.payload) as NotesResponse;
    }

    syncAuthState(viewer ?? notes?.viewer ?? null);
    lastRefresh = new Date().toLocaleTimeString();
  }

  async function tryWrite() {
    writeResponse = null;
    writeProbe = { ...writeProbe, state: "loading", detail: "Request in flight.", payload: "", status: null };

    const nextWriteProbe = await runProbe(writeProbe, "/api/v1/notes", {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify({ title: `validation-${authState.authType ?? "anonymous"}` }),
    });

    writeProbe = nextWriteProbe;

    if (writeProbe.state === "success" && writeProbe.payload) {
      writeResponse = JSON.parse(writeProbe.payload) as WriteResponse;
    }
  }

  onMount(async () => {
    await refreshDashboard();
  });
</script>

<main class="shell">
  <Card class="hero-card">
    <div>
      <SectionLabel>app.smoke</SectionLabel>
      <h1>Protected notes validation target</h1>
      <p class="lede">The app still trusts upstream identity headers, but now the notes demo makes auth mode, scope expectations, and gateway denials visible in one place.</p>
    </div>

    <div class="hero-metrics">
      <MetricTile label="Environment" value={envLabel} />
      <MetricTile label="Health" tone={metricTone(healthText)}>
        <strong class={metricTone(healthText)}>{healthText}</strong>
      </MetricTile>
      <MetricTile label="Auth mode" value={authModeLabel} />
      <MetricTile label="Viewer" value={viewerLabel} />
    </div>
  </Card>

  <section class="dashboard">
    <Card class="identity-card">
      <div class="section-head">
        <CardHeader>
          <SectionLabel>Viewer</SectionLabel>
          <h2>Trusted identity snapshot</h2>
        </CardHeader>
        <StatusPill tone={probeTone(viewerProbe.state)}>{probeLabel(viewerProbe.state)}</StatusPill>
      </div>

      <div class="identity-grid">
        <div>
          <dt>Subject</dt>
          <dd><code>{authState.subject ?? "anonymous"}</code></dd>
        </div>
        <div>
          <dt>Auth type</dt>
          <dd>{authState.authType ?? "none"}</dd>
        </div>
        <div>
          <dt>Scopes</dt>
          <dd>{authState.scopes.join(", ") || "none"}</dd>
        </div>
        <div>
          <dt>Scope check</dt>
          <dd>{hasAllScopes(notesReadScopes) ? "Can satisfy notes read contract" : "Missing notes:read in viewer context"}</dd>
        </div>
      </div>

      <div class="scope-row">
        <InfoChip tone={authModeTone}>{authState.authType ? `auth:${authState.authType}` : "auth:none"}</InfoChip>
        <InfoChip tone={hasAllScopes(notesReadScopes) ? "success" : "danger"}>notes:read</InfoChip>
        <InfoChip tone={hasAllScopes(notesWriteScopes) ? "success" : "danger"}>notes:write</InfoChip>
      </div>

      <p class="caption">{viewerProbe.detail}</p>
      <p class="caption">Health: <strong>{healthDetail}</strong> · Last refresh: <strong>{lastRefresh}</strong></p>

      {#if viewerProbe.payload}
        <pre>{viewerProbe.payload}</pre>
      {/if}
    </Card>

    <Card class="checks-card">
      <div class="section-head">
        <CardHeader>
          <SectionLabel>Validation</SectionLabel>
          <h2>Gateway outcome checks</h2>
        </CardHeader>
        <div class="action-row">
          <Button size="lg" variant="secondary" onclick={refreshDashboard}>refresh checks</Button>
          <Button size="lg" onclick={tryWrite}>attempt write</Button>
        </div>
      </div>

      <article class="probe-card">
        <div class="probe-head">
          <div>
            <p class="probe-label">{notesProbe.method} {notesProbe.path}</p>
            <h3>Notes read contract</h3>
          </div>
          <span class={probeBadgeClass(notesProbe.state)}>{probeLabel(notesProbe.state)}</span>
        </div>
        <p class="probe-copy">Expected signal: `notes:read` should allow the request, while gateway denial should show up as `401` or `403` here.</p>
        <div class="probe-meta">
          <InfoChip tone={hasAllScopes(notesReadScopes) ? "success" : "danger"}>viewer has notes:read</InfoChip>
          <InfoChip tone={notesProbe.state === "success" ? "success" : notesProbe.state === "denied" ? "danger" : "default"}>http {statusLabel(notesProbe.status)}</InfoChip>
        </div>
        <p class="caption">{notesProbe.detail}</p>
      </article>

      <article class="probe-card">
        <div class="probe-head">
          <div>
            <p class="probe-label">{writeProbe.method} {writeProbe.path}</p>
            <h3>Notes write contract</h3>
          </div>
          <span class={probeBadgeClass(writeProbe.state)}>{probeLabel(writeProbe.state)}</span>
        </div>
        <p class="probe-copy">Use this to prove session-vs-token and scope policy. If the gateway blocks write access, the business app should not return the smoke response.</p>
        <div class="probe-meta">
          <InfoChip tone={hasAllScopes(notesWriteScopes) ? "success" : "danger"}>viewer has notes:write</InfoChip>
          <InfoChip tone={writeProbe.state === "success" ? "success" : writeProbe.state === "denied" ? "danger" : "default"}>http {statusLabel(writeProbe.status)}</InfoChip>
        </div>
        <p class="caption">{writeProbe.detail}</p>
        {#if writeResponse}
          <p class="caption">App response: <strong>{writeResponse.message}</strong></p>
        {/if}
      </article>

      {#if writeProbe.payload}
        <pre>{writeProbe.payload}</pre>
      {/if}
    </Card>
  </section>

  <Card class="notes-card">
    <div class="section-head">
      <CardHeader>
        <SectionLabel>Notes</SectionLabel>
        <h2>Protected sample payload</h2>
      </CardHeader>
      <StatusPill tone={probeTone(notesProbe.state)}>{notesVisibility}</StatusPill>
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
      <pre>{notesProbe.payload}</pre>
    {:else if notesProbe.state === "denied"}
      <div class="denial-card">
        <h3>Notes payload blocked by gateway</h3>
        <p>The denial response stays visible so you can confirm that auth policy stopped the request before business data was returned.</p>
      </div>
      {#if notesProbe.payload}
        <pre>{notesProbe.payload}</pre>
      {/if}
    {:else}
      <p>{notesProbe.detail}</p>
    {/if}
  </Card>
</main>

<style>
  :global(:root) {
    --lt-page-bg: #eef3f7;
    --lt-color-primary: #6f5a2f;
    --lt-color-primary-hover: #594822;
    --lt-color-on-primary: #fffdf8;
    --lt-color-surface: rgba(255, 255, 255, 0.88);
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

  .hero-metrics,
  .note-list {
    display: grid;
    gap: 0.75rem;
  }

  .lede {
    margin: 0;
    color: #5d6774;
    max-width: 54rem;
  }

  dt,
  .note-id,
  .caption,
  .probe-label {
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

  .section-head,
  .probe-head,
  .action-row,
  .probe-meta,
  .scope-row {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .section-head,
  .probe-head {
    justify-content: space-between;
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

  h1,
  h2,
  h3,
  p {
    margin: 0;
  }

  h1,
  h2 {
    margin-bottom: 0.75rem;
  }

  pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-size: 0.92rem;
    padding: 1rem;
    border-radius: 0.9rem;
    background: rgba(247, 243, 234, 0.95);
    border: 1px solid rgba(29, 39, 51, 0.08);
  }

  code {
    font-family: "IBM Plex Mono", monospace;
  }

  .identity-grid {
    display: grid;
    gap: 0.95rem;
    margin-bottom: 1rem;
  }

  .identity-grid dd {
    margin: 0.2rem 0 0;
  }

  .checks-card,
  .notes-card {
    display: grid;
    gap: 1rem;
  }

  .probe-card,
  .denial-card,
  .note-item {
    padding: 1rem;
    border-radius: 0.9rem;
    background: #f7f3ea;
    border: 1px solid rgba(29, 39, 51, 0.08);
  }

  .probe-copy {
    margin-bottom: 0.75rem;
    color: var(--lt-color-text-muted);
  }

  .probe-badge {
    border-radius: 999px;
    padding: 0.35rem 0.7rem;
    border: 1px solid var(--lt-color-border);
    color: var(--lt-color-text-subtle);
    background: rgba(255, 255, 255, 0.8);
    font-size: 0.82rem;
  }

  .probe-badge--success {
    background: var(--lt-color-success-surface);
    color: var(--lt-color-success);
    border-color: var(--lt-color-success-border);
  }

  .probe-badge--denied,
  .probe-badge--error {
    background: var(--lt-color-danger-surface);
    color: var(--lt-color-danger);
    border-color: var(--lt-color-danger-border);
  }

  .probe-badge--loading {
    background: rgba(111, 90, 47, 0.08);
    color: #6f5a2f;
  }

  .denial-card h3 {
    margin-bottom: 0.45rem;
  }

  @media (max-width: 780px) {
    .dashboard,
    :global(.hero-card) {
      grid-template-columns: 1fr;
    }

    .shell {
      padding: 1rem;
    }
  }
</style>
