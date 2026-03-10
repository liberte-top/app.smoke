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

  let healthText = "loading";
  let viewer: Viewer | null = null;
  let notes: NotesResponse | null = null;
  let errorText = "";

  onMount(async () => {
    try {
      const [health, viewerRes, notesRes] = await Promise.all([
        http.get("/api/v1/health"),
        http.get("/api/v1/viewer", {
          headers: {
            "x-auth-subject": "demo-user",
            "x-auth-type": "session",
            "x-auth-scopes": "notes:read profile:read",
          },
        }),
        http.get("/api/v1/notes", {
          headers: {
            "x-auth-subject": "demo-user",
            "x-auth-type": "session",
            "x-auth-scopes": "notes:read profile:read",
          },
        }),
      ]);

      healthText = health.data.status;
      viewer = viewerRes.data;
      notes = notesRes.data;
    } catch (error) {
      healthText = "unreachable";
      errorText = error instanceof Error ? error.message : "request failed";
    }
  });
</script>

<main>
  <section class="card">
    <h1>app.smoke web</h1>
    <p>Environment: <strong>{envLabel}</strong></p>
    <p>
      Business API health:
      <strong>{healthText}</strong>
    </p>
    <p>This sample assumes auth and scope enforcement happen upstream.</p>
  </section>

  <section class="card">
    <h2>Gateway-provided identity</h2>
    {#if viewer}
      <pre>{JSON.stringify(viewer, null, 2)}</pre>
    {:else}
      <p>{errorText || "Loading viewer context..."}</p>
    {/if}
  </section>

  <section class="card">
    <h2>Sample business payload</h2>
    {#if notes}
      <pre>{JSON.stringify(notes, null, 2)}</pre>
    {:else}
      <p>{errorText || "Loading notes..."}</p>
    {/if}
  </section>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: "Iowan Old Style", "Palatino Linotype", serif;
    background: linear-gradient(180deg, #f6f3ea 0%, #e8eef3 100%);
    color: #1d2733;
  }

  main {
    min-height: 100vh;
    padding: 2rem;
    display: grid;
    gap: 1rem;
  }

  .card {
    background: rgba(255, 255, 255, 0.86);
    border: 1px solid rgba(29, 39, 51, 0.12);
    border-radius: 1rem;
    padding: 1.25rem;
    box-shadow: 0 12px 30px rgba(29, 39, 51, 0.08);
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
</style>
