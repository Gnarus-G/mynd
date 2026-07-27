<script lang="ts">
  import { onMount, tick } from "svelte";
  import QRCode from "qrcode";
  import { registerSW } from "virtual:pwa-register";
  import {
    addTodo,
    apiError,
    cleanTodos,
    deleteTodo,
    getWebUrl,
    load,
    loading,
    moveBelow,
    moveDown,
    moveUp,
    mutating,
    removeTodo,
    todos,
    type Todo,
  } from "$lib/store";

  let message = "";
  let online = true;
  let list: HTMLOListElement;
  let shareDialog: HTMLDialogElement;
  let qrCanvas: HTMLCanvasElement;
  let shareUrl = "";
  let copied = false;
  let updateAvailable = false;
  let draggedId = "";
  let mobileUrl: string | null = null;
  let installPrompt: InstallPromptEvent | null = null;
  let showInstallInvite = false;

  type InstallPromptEvent = Event & {
    prompt: () => Promise<void>;
    userChoice: Promise<{ outcome: "accepted" | "dismissed" }>;
  };

  const updateServiceWorker = registerSW({
    immediate: true,
    onNeedRefresh() {
      updateAvailable = true;
    },
  });

  onMount(() => {
    online = navigator.onLine;
    load();
    getWebUrl().then((url) => (mobileUrl = url)).catch(() => undefined);

    const standalone = window.matchMedia("(display-mode: standalone)").matches;
    showInstallInvite = !standalone && !sessionStorage.getItem("mynd-install-dismissed");

    const offerInstall = (event: Event) => {
      event.preventDefault();
      installPrompt = event as InstallPromptEvent;
      showInstallInvite = true;
    };
    const installed = () => {
      installPrompt = null;
      showInstallInvite = false;
    };

    const setOnline = () => {
      online = navigator.onLine;
      if (online) load();
    };
    window.addEventListener("beforeinstallprompt", offerInstall);
    window.addEventListener("appinstalled", installed);
    window.addEventListener("online", setOnline);
    window.addEventListener("offline", setOnline);
    return () => {
      window.removeEventListener("beforeinstallprompt", offerInstall);
      window.removeEventListener("appinstalled", installed);
      window.removeEventListener("online", setOnline);
      window.removeEventListener("offline", setOnline);
    };
  });

  async function installApp() {
    if (!installPrompt) return;
    await installPrompt.prompt();
    await installPrompt.userChoice;
    installPrompt = null;
    showInstallInvite = false;
  }

  function dismissInstall() {
    sessionStorage.setItem("mynd-install-dismissed", "true");
    showInstallInvite = false;
  }

  async function submit() {
    const submitted = message;
    if (!submitted.trim()) return;
    try {
      await addTodo(submitted);
      message = "";
      await tick();
      list?.lastElementChild?.scrollIntoView({ behavior: "smooth", block: "nearest" });
    } catch {
      // The store exposes the actionable API error beside the form.
    }
  }

  async function confirmDelete(todo: Todo) {
    if (!confirm(`Permanently delete “${todo.message}”?`)) return;
    await deleteTodo(todo.id).catch(() => undefined);
  }

  async function openMobileAccess() {
    if (!mobileUrl) return;
    shareUrl = mobileUrl;
    copied = false;
    shareDialog.showModal();
    await tick();
    await QRCode.toCanvas(qrCanvas, shareUrl, {
      width: 224,
      margin: 2,
      color: { dark: "#17221c", light: "#f3f0e8" },
      errorCorrectionLevel: "M",
    });
  }

  async function copyLink() {
    await navigator.clipboard.writeText(shareUrl);
    copied = true;
  }

  async function shareLink() {
    if (!navigator.share) return copyLink();
    await navigator.share({ title: "Mynd", text: "Open Mynd on this device", url: shareUrl });
  }

  function dropBelow(targetId: string) {
    if (draggedId && draggedId !== targetId) {
      moveBelow(draggedId, targetId).catch(() => undefined);
    }
    draggedId = "";
  }
</script>

<svelte:window on:focus={load} />

<svelte:head>
  <meta property="og:title" content="Mynd" />
  <meta property="og:description" content="A fast, private todo capture tool." />
</svelte:head>

<div class="app-shell">
  <header class="topbar">
    <a class="brand" href="/" aria-label="Mynd home">
      <img src="/icons/logo.svg" alt="" />
      <span>mynd</span>
    </a>

    <div class="topbar-actions">
      <span class:offline={!online} class="connection">
        <i></i>{online ? "connected" : "offline"}
      </span>
      <button class="share-button" type="button" on:click={openMobileAccess} disabled={!mobileUrl} title={mobileUrl ? "Share Mynd with another tailnet device" : "Configure a Tailscale HTTPS URL first"}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 3H5a2 2 0 0 0-2 2v2m14-4h2a2 2 0 0 1 2 2v2M7 21H5a2 2 0 0 1-2-2v-2m14 4h2a2 2 0 0 0 2-2v-2M8 8h3v3H8zm5 0h3v3h-3zm-5 5h3v3H8zm7 0h1v1h-1zm-2 2h1v1h-1zm2 1h1v-1h-1z" /></svg>
        <span>Mobile</span>
      </button>
    </div>
  </header>

  {#if showInstallInvite}
    <aside class="install-invite" aria-label="Install Mynd">
      <img src="/icons/logo.svg" alt="" />
      <div>
        <strong>Install Mynd</strong>
        <span>{installPrompt ? "Open your list without the browser chrome." : "Use your browser menu, then choose Install app or Add to Home Screen."}</span>
      </div>
      {#if installPrompt}
        <button class="install-action" type="button" on:click={installApp}>Install</button>
      {/if}
      <button class="install-dismiss" type="button" aria-label="Dismiss install invitation" on:click={dismissInstall}>×</button>
    </aside>
  {/if}

  <main>
    <section class="workspace" aria-labelledby="ledger-title">
      <div class="capture-panel">
        <form on:submit|preventDefault={submit}>
          <label for="new-todo">Quick capture</label>
          <div class="capture-row">
            <input
              id="new-todo"
              name="todo"
              bind:value={message}
              autocomplete="off"
              placeholder="What needs doing?"
              disabled={$mutating || !online}
            />
            <button type="submit" disabled={$mutating || !message.trim() || !online}>
              <span>{$mutating ? "Saving" : "Add"}</span>
              <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h14m-5-5 5 5-5 5" /></svg>
            </button>
          </div>
        </form>
        {#if $apiError}
          <p class="error-message" role="alert">{$apiError}</p>
        {:else if !online}
          <p class="error-message" role="status">Mynd needs a live connection to your home server.</p>
        {/if}
      </div>

      <div class="ledger-heading">
        <h2 id="ledger-title">Todos</h2>
        {#if $todos.some((todo) => todo.done)}
          <button class="text-button danger" type="button" on:click={() => cleanTodos()} disabled={$mutating}>
            Clear completed
          </button>
        {/if}
      </div>

      {#if $loading}
        <div class="loading-state" aria-live="polite"><i></i><span>Loading</span></div>
      {:else if !$todos.length}
        <div class="empty-state">
          <img src="/icons/logo.svg" alt="" />
          <p>Nothing pending.</p>
          <small>Add a thought above and get on with your day.</small>
        </div>
      {:else}
        <ol class="todo-list" bind:this={list}>
          {#each $todos as todo, index (todo.id)}
            <li
              class:done={todo.done}
              draggable="true"
              on:dragstart={() => (draggedId = todo.id)}
              on:dragover|preventDefault
              on:drop={() => dropBelow(todo.id)}
            >
              <button
                class="complete-button"
                class:checked={todo.done}
                type="button"
                aria-label={todo.done ? `Mark ${todo.message} active` : `Complete ${todo.message}`}
                on:click={() => removeTodo(todo.id)}
                disabled={$mutating}
              >
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 4 4L19 6" /></svg>
              </button>

              <div class="todo-copy">
                <p>{todo.message}</p>
                <time datetime={todo.created_at}>{new Date(todo.created_at).toLocaleString([], { dateStyle: "medium", timeStyle: "short" })}</time>
              </div>

              <div class="todo-actions">
                <button type="button" aria-label={`Move ${todo.message} up`} on:click={() => moveUp(todo.id)} disabled={index === 0 || $mutating}>
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m7 14 5-5 5 5" /></svg>
                </button>
                <button type="button" aria-label={`Move ${todo.message} down`} on:click={() => moveDown(todo.id)} disabled={index === $todos.length - 1 || $mutating}>
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m7 10 5 5 5-5" /></svg>
                </button>
                <button class="delete-button" type="button" aria-label={`Delete ${todo.message}`} on:click={() => confirmDelete(todo)} disabled={$mutating}>
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 8v10m4-10v10m4-10v10M5 5h14M9 5V3h6v2m3 0-1 16H7L6 5" /></svg>
                </button>
              </div>
            </li>
          {/each}
        </ol>
      {/if}
    </section>
  </main>
</div>

{#if updateAvailable}
  <aside class="update-banner" aria-live="polite">
    <span>A fresh build is ready.</span>
    <button type="button" on:click={() => updateServiceWorker(true)}>Update now</button>
  </aside>
{/if}

<dialog class="share-dialog" bind:this={shareDialog}>
  <div class="dialog-card">
    <button class="dialog-close" type="button" aria-label="Close" on:click={() => shareDialog.close()}>×</button>
    <p class="eyebrow">Pocket access</p>
    <h2>Take Mynd with you.</h2>
    <p class="dialog-intro">Connect Tailscale on your phone, then scan this private link.</p>
    <div class="qr-frame"><canvas bind:this={qrCanvas} aria-label="QR code for the Mynd mobile URL"></canvas></div>
    <code>{shareUrl}</code>
    <div class="dialog-actions">
      <button class="primary" type="button" on:click={shareLink}>Share link</button>
      <button type="button" on:click={copyLink}>{copied ? "Copied" : "Copy URL"}</button>
    </div>
    <p class="install-note"><strong>Install:</strong> use “Add to Home Screen” on iOS or “Install app” in Chrome after opening the link.</p>
  </div>
</dialog>
