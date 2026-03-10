<script lang="ts">
  import { onMount } from "svelte";
  import axios from "axios";
  import { createAuth } from "@liberte-top/auth";

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

  const auth = createAuth({ authDomain: "auth.liberte.top" });

  let healthText = "loading";
  let viewer: Viewer | null = null;
  let notes: NotesResponse | null = null;
  let errorText = "";
  let authState = auth.snapshot();

  onMount(async () => {
    try {
      authState = await auth.refresh();

      const [health, viewerRes, notesRes] = await Promise.all([
        http.get("/api/v1/health"),
        http.get("/api/v1/viewer"),
        http.get("/api/v1/notes"),
      ]);

      healthText = health.data.status;
      viewer = viewerRes.data;
      notes = notesRes.data;
      authState = auth.snapshot();
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
      <div class="scope-demo">
        <span class:ok={auth.scopes.any(["notes:read"])}>notes:read</span>
        <span class:ok={auth.scopes.all(["notes:read", "profile:read"])}>
          notes:read + profile:read
        </span>
        <button disabled={!auth.scopes.all(["notes:write"])}>write action</button>
      </div>
      <pre>{JSON.stringify(authState, null, 2)}</pre>
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
</style>
