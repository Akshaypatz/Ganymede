<script lang="ts">
  import { onMount } from "svelte";
  import { items, addToast } from "../stores/app";
  import { listItems, createItem, updateItem, deleteItem } from "../api";
  import type { Item } from "../types";

  const stages = [
    { key: "brainstorm", label: "Brainstorm", color: "#8b5cf6", bg: "rgba(139,92,246,.12)" },
    { key: "exploring", label: "Exploring", color: "#3b82f6", bg: "rgba(59,130,246,.12)" },
    { key: "planned", label: "Planned", color: "#f97316", bg: "rgba(249,115,22,.12)" },
    { key: "ready", label: "Ready to Start", color: "#22c55e", bg: "rgba(34,197,94,.12)" },
  ];

  let showCreateModal = false;
  let createStage = "brainstorm";
  let createTitle = "";
  let createBody = "";
  let createWhy = "";
  let createError = "";
  let editingItem: Item | null = null;
  let editTitle = "";
  let editBody = "";
  let editStage = "";
  let dragItem: Item | null = null;
  let dragOverCol = "";

  onMount(async () => {
    try { const it = await listItems({}); items.set(it); } catch {}
  });

  $: ideas = $items.filter((i) => i.type === "idea" && i.status !== "done");

  // Reactive per-stage grouping — re-runs whenever ideas changes
  $: ideasByStage = stages.reduce((acc, s) => {
    acc[s.key] = ideas.filter((i) => i.status === s.key);
    return acc;
  }, {} as Record<string, Item[]>);

  async function handleCreate() {
    createError = "";
    if (!createTitle.trim()) { createError = "Title is required."; return; }
    const title = createTitle.trim();
    const why = createWhy.trim();
    const notes = createBody.trim();
    const body = why ? `**Why:** ${why}${notes ? '\n\n' + notes : ''}` : notes;
    showCreateModal = false;
    createTitle = ""; createBody = ""; createWhy = "";
    const item = await createItem({ title, body, type: "idea", status: createStage as any, priority: "p3" });
    items.update((list) => [item, ...list]);
    addToast(`Idea added: "${item.title.slice(0, 40)}"`, "✓");
  }

  function openEdit(item: Item) {
    editingItem = item;
    editTitle = item.title;
    editBody = item.body;
    editStage = item.status;
  }

  async function saveEdit() {
    if (!editingItem) return;
    const updated = await updateItem({
      id: editingItem.id,
      title: editTitle,
      body: editBody,
      status: editStage as any,
    });
    items.update((list) => list.map((i) => (i.id === updated.id ? { ...updated, tags: i.tags, project_name: i.project_name } : i)));
    editingItem = null;
    addToast(`Updated: "${editTitle.slice(0, 40)}"`, "✓");
  }

  async function archiveIdea(item: Item) {
    await updateItem({ id: item.id, status: "done" });
    items.update((list) => list.map((i) => (i.id === item.id ? { ...i, status: "done" as const } : i)));
    addToast(`Archived: "${item.title.slice(0, 40)}"`, "✓");
  }

  async function removeIdea(item: Item) {
    await deleteItem(item.id);
    items.update((list) => list.filter((i) => i.id !== item.id));
    addToast(`Removed: "${item.title.slice(0, 40)}"`, "✗");
  }

  // Drag and drop
  function handleDragStart(e: DragEvent, item: Item) {
    dragItem = item;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", item.id);
    }
  }

  function handleDragOver(e: DragEvent, stageKey: string) {
    e.preventDefault();
    dragOverCol = stageKey;
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
  }

  function handleDragLeave() {
    dragOverCol = "";
  }

  async function handleDrop(e: DragEvent, stageKey: string) {
    e.preventDefault();
    dragOverCol = "";
    if (!dragItem || dragItem.status === stageKey) { dragItem = null; return; }
    const item = dragItem;
    dragItem = null;
    await updateItem({ id: item.id, status: stageKey as any });
    items.update((list) => list.map((i) => (i.id === item.id ? { ...i, status: stageKey as any } : i)));
    const stageLabel = stages.find((s) => s.key === stageKey)?.label || stageKey;
    addToast(`Moved to ${stageLabel}`, "→");
  }
</script>

<svelte:window on:keydown={(e) => { if (e.key === 'Escape') { editingItem = null; showCreateModal = false; } }} />

<div class="page-header">
  <div class="page-hdr-row">
    <div>
      <div class="page-title">Idea Board</div>
      <div class="page-sub">Brainstorm and plan upcoming projects and sprints</div>
    </div>
  </div>
</div>

<div class="board">
  {#each stages as stage}
    <div
      class="board-col"
      class:drag-over={dragOverCol === stage.key}
      on:dragover={(e) => handleDragOver(e, stage.key)}
      on:dragleave={handleDragLeave}
      on:drop={(e) => handleDrop(e, stage.key)}
      role="list"
    >
      <div class="board-col-hdr" style="background:{stage.bg};color:{stage.color};border-color:{stage.color}33">
        <button
          class="col-hdr-add"
          style="color:{stage.color}"
          on:click|stopPropagation={() => { showCreateModal = true; createStage = stage.key; createTitle = ''; createBody = ''; createWhy = ''; createError = ''; }}
          title="Add idea"
        >
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        </button>
        <span class="col-hdr-label">{stage.label}</span>
        <span class="col-count" style="background:{stage.color}22">{ideasByStage[stage.key]?.length ?? 0}</span>
      </div>
      <div class="board-col-body">
        {#each ideasByStage[stage.key] ?? [] as item (item.id)}
          <div
            class="idea-card"
            draggable="true"
            on:dragstart={(e) => handleDragStart(e, item)}
            on:click={() => openEdit(item)}
            role="listitem"
            tabindex="0"
            on:keydown={(e) => e.key === 'Enter' && openEdit(item)}
          >
            <div class="idea-title">{item.title}</div>
            {#if item.body}
              <div class="idea-body">{item.body.slice(0, 80)}{item.body.length > 80 ? "…" : ""}</div>
            {/if}
            <div class="idea-footer">
              <span class="idea-date">{new Date(item.created_at).toLocaleDateString("en-US", { month: "short", day: "numeric" })}</span>
              <button class="idea-archive" on:click|stopPropagation={() => archiveIdea(item)} title="Archive">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
              </button>
            </div>
          </div>
        {:else}
          <div class="board-empty">Drop here or click + to add</div>
        {/each}
      </div>
    </div>
  {/each}
</div>

<!-- Edit modal -->
{#if editingItem}
  <div class="overlay" on:click|self={() => (editingItem = null)} role="dialog">
    <div class="modal edit-modal">
      <div class="modal-header">
        <span class="modal-title">Edit Idea</span>
        <div class="modal-header-actions">
          <button class="btn-sm danger" on:click={() => { if (editingItem) removeIdea(editingItem); editingItem = null; }} title="Delete">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-2 14H7L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/></svg>
          </button>
          <button class="btn-icon" on:click={() => (editingItem = null)}>✕</button>
        </div>
      </div>
      <div class="modal-body">
        <input bind:value={editTitle} class="edit-title-input" placeholder="Idea title…" />
        <div class="field-label">Stage</div>
        <div class="stage-pill-group">
          {#each stages as s}
            <button
              class="stage-choice-pill"
              class:active={editStage === s.key}
              style={editStage === s.key ? `background:${s.bg};border-color:${s.color};color:${s.color}` : ""}
              on:click={() => editStage = s.key}
            >{s.label}</button>
          {/each}
        </div>
        <div class="field-label">Notes</div>
        <textarea bind:value={editBody} class="field-textarea" rows="4" placeholder="Context, links, next steps…"></textarea>
      </div>
      <div class="modal-footer">
        <button class="btn btn-ghost" on:click={() => (editingItem = null)}>Cancel</button>
        <button class="btn btn-primary" on:click={saveEdit}>Save</button>
      </div>
    </div>
  </div>
{/if}

<!-- Create Idea Modal -->
{#if showCreateModal}
  <div class="overlay" on:click|self={() => showCreateModal = false} role="dialog">
    <div class="modal create-modal">
      <div class="modal-header">
        <div class="modal-header-left">
          <span class="create-icon">💡</span>
          <span class="modal-title">New Idea</span>
        </div>
        <button class="btn-icon" on:click={() => showCreateModal = false}>✕</button>
      </div>
      <div class="modal-body">
        {#if createError}
          <div class="form-error">{createError}</div>
        {/if}
        <input
          bind:value={createTitle}
          class="idea-create-title"
          placeholder="What's the idea?"
          autofocus
          on:keydown={(e) => { if (e.key === 'Enter') handleCreate(); }}
        />
        <div class="field-col-group">
          <div class="field-label">Stage</div>
          <div class="stage-pill-group">
            {#each stages as s}
              <button
                class="stage-choice-pill"
                class:active={createStage === s.key}
                style={createStage === s.key ? `background:${s.bg};border-color:${s.color};color:${s.color}` : ""}
                on:click={() => createStage = s.key}
              >{s.label}</button>
            {/each}
          </div>
        </div>
        <div class="field-col-group">
          <div class="field-label">Why this matters <span class="opt-hint">(optional)</span></div>
          <input bind:value={createWhy} class="create-field-input" placeholder="The problem it solves or opportunity it unlocks…" />
        </div>
        <div class="field-col-group">
          <div class="field-label">Notes</div>
          <textarea bind:value={createBody} class="field-textarea" rows="3" placeholder="Context, links, related ideas, next steps…"></textarea>
        </div>
      </div>
      <div class="modal-footer create-modal-footer">
        <span class="create-hint">💬 A clear “why” helps turn ideas into initiatives</span>
        <div style="flex:1"></div>
        <button class="btn btn-ghost" on:click={() => showCreateModal = false}>Cancel</button>
        <button class="btn btn-primary" on:click={handleCreate}>Add Idea</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .page-header { margin-bottom: 14px; }
  .page-hdr-row { display: flex; align-items: flex-start; justify-content: space-between; }
  .page-title { font-size: 17px; font-weight: 750; letter-spacing: -0.03em; }
  .page-sub { font-size: 12px; color: var(--fg-3); margin-top: 3px; }

  .col-add-btn {
    display: none;
  }

  .inline-create {
    display: flex; flex-direction: column; gap: 6px;
    background: var(--bg); border: 1px solid var(--orange); border-radius: var(--r-lg);
    padding: 10px 12px; box-shadow: 0 2px 8px rgba(0,0,0,.12);
  }
  .inline-input {
    width: 100%; padding: 5px 0; background: transparent;
    border: none; border-bottom: 1px solid var(--border);
    font-size: 13px; font-weight: 550; color: var(--fg); outline: none;
  }
  .inline-input:focus { border-color: var(--orange); }
  .inline-input::placeholder { color: var(--fg-4); font-weight: 400; }
  .inline-textarea {
    width: 100%; padding: 4px 0; background: transparent;
    border: none; font-size: 11px; color: var(--fg-2); resize: none;
    font-family: inherit; outline: none; line-height: 1.5;
  }
  .inline-textarea::placeholder { color: var(--fg-4); }
  .inline-actions { display: flex; gap: 6px; padding-top: 2px; }
  .inline-add-btn {
    padding: 4px 12px; font-size: 11px; font-weight: 600; border-radius: 6px; cursor: pointer;
    background: var(--orange); border: 1px solid var(--orange); color: #fff; transition: opacity 100ms;
  }
  .inline-add-btn:hover { opacity: 0.88; }
  .inline-cancel-btn {
    padding: 4px 10px; font-size: 11px; font-weight: 500; border-radius: 6px; cursor: pointer;
    background: none; border: 1px solid var(--border); color: var(--fg-3); transition: all 100ms;
  }
  .inline-cancel-btn:hover { border-color: var(--border-2); color: var(--fg); }

  /* Board */
  .board {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px;
    align-items: start;
  }
  .board-col { display: flex; flex-direction: column; min-height: 200px; transition: all 150ms; }
  .board-col.drag-over { transform: scale(1.01); }
  .board-col-hdr {
    display: flex; align-items: center; gap: 6px;
    padding: 6px 8px 6px 6px; border-radius: var(--r-lg) var(--r-lg) 0 0;
    font-size: 11px; font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase;
    border: 1px solid; border-bottom: none;
  }
  .col-hdr-add {
    width: 20px; height: 20px; display: flex; align-items: center; justify-content: center;
    border-radius: 4px; background: none; cursor: pointer; opacity: 0.7;
    transition: opacity 120ms, background 120ms; flex-shrink: 0;
  }
  .col-hdr-add:hover { opacity: 1; background: rgba(255,255,255,.12); }
  .col-hdr-label { flex: 1; }
  .col-count {
    font-size: 10px; font-weight: 800; padding: 1px 6px; border-radius: 99px; min-width: 18px; text-align: center;
  }
  .board-col-body {
    flex: 1; display: flex; flex-direction: column; gap: 6px;
    padding: 8px; border: 1px solid var(--border); border-top: none;
    border-radius: 0 0 var(--r-lg) var(--r-lg); background: var(--surface);
    min-height: 80px;
  }
  .board-col.drag-over .board-col-body { background: var(--surface-2); border-color: var(--border-2); }
  .board-empty { font-size: 11px; color: var(--fg-4); text-align: center; padding: 20px 0; }

  .idea-card {
    background: var(--bg); border: 1px solid var(--border); border-radius: var(--r-lg);
    padding: 10px 12px; cursor: grab; transition: all 130ms;
    display: flex; flex-direction: column; gap: 6px;
  }
  .idea-card:hover { border-color: var(--border-2); box-shadow: var(--shadow); }
  .idea-card:active { cursor: grabbing; }
  .idea-title { font-size: 12px; font-weight: 560; color: var(--fg); line-height: 1.4; }
  .idea-body { font-size: 11px; color: var(--fg-3); line-height: 1.4; }
  .idea-footer { display: flex; align-items: center; justify-content: space-between; }
  .idea-date { font-size: 9px; color: var(--fg-4); }
  .idea-archive {
    width: 20px; height: 20px; display: flex; align-items: center; justify-content: center;
    color: var(--fg-4); border: 1px solid var(--border); border-radius: 4px;
    background: none; cursor: pointer; transition: all 100ms;
  }
  .idea-archive:hover { background: var(--green-bg); border-color: var(--green); color: var(--green); }

  /* Edit Modal */
  .edit-modal { width: 460px; max-height: 80vh; overflow-y: auto; }
  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 18px; border-bottom: 1px solid var(--border);
  }
  .modal-header-actions { display: flex; align-items: center; gap: 6px; }
  .modal-title { font-size: 13px; font-weight: 700; }
  .modal-body { padding: 18px; display: flex; flex-direction: column; gap: 12px; }
  .modal-footer { padding: 12px 18px; border-top: 1px solid var(--border); display: flex; gap: 8px; justify-content: flex-end; }

  .edit-title-input {
    width: 100%; padding: 8px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r-lg);
    font-size: 13px; font-weight: 600; color: var(--fg);
  }
  .edit-title-input:focus { border-color: var(--orange); outline: none; }

  .field-label { font-size: 10px; font-weight: 700; color: var(--fg-4); text-transform: uppercase; letter-spacing: 0.06em; }

  .stage-pill-group { display: flex; gap: 5px; flex-wrap: wrap; }
  .stage-choice-pill {
    padding: 5px 12px; border-radius: 99px; font-size: 11px; font-weight: 600; cursor: pointer;
    border: 1px solid var(--border); background: var(--surface-2); color: var(--fg-3);
    transition: all 100ms;
  }
  .stage-choice-pill:hover { border-color: var(--border-2); color: var(--fg); }
  .stage-choice-pill.active { }

  .field-textarea {
    width: 100%; padding: 8px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r-lg); font-size: 12px; color: var(--fg);
    resize: vertical; font-family: inherit; line-height: 1.5;
  }
  .field-textarea:focus { border-color: var(--orange); outline: none; }

  .btn-sm {
    padding: 4px 8px; border-radius: 5px; font-size: 10px; display: flex; align-items: center; gap: 3px;
    border: 1px solid var(--border); background: none; color: var(--fg-3); cursor: pointer; transition: all 100ms;
  }
  .btn-sm.danger:hover { background: var(--p0-bg); border-color: var(--p0); color: var(--p0); }

  /* Create Idea Modal */
  .create-modal { width: 500px; max-height: 85vh; overflow-y: auto; }
  .modal-header-left { display: flex; align-items: center; gap: 8px; }
  .create-icon { font-size: 20px; line-height: 1; }
  .opt-hint { font-size: 10px; color: var(--fg-4); font-weight: 400; text-transform: none; letter-spacing: 0; margin-left: 3px; }
  .field-col-group { display: flex; flex-direction: column; gap: 6px; }
  .idea-create-title {
    width: 100%; padding: 10px 12px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r-lg);
    font-size: 15px; font-weight: 650; color: var(--fg); letter-spacing: -0.02em;
    transition: border-color 120ms;
  }
  .idea-create-title:focus { border-color: var(--orange); outline: none; box-shadow: 0 0 0 3px rgba(249,115,22,.08); }
  .idea-create-title::placeholder { color: var(--fg-4); font-weight: 400; }
  .create-field-input {
    padding: 8px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r);
    font-size: 12px; color: var(--fg); width: 100%; font-family: inherit;
  }
  .create-field-input:focus { border-color: var(--orange); outline: none; }
  .create-modal-footer { display: flex; align-items: center; gap: 8px; }
  .create-hint { font-size: 11px; color: var(--fg-4); font-style: italic; }
  .form-error {
    padding: 7px 10px; background: rgba(239,68,68,.08); border: 1px solid rgba(239,68,68,.25);
    border-radius: var(--r); font-size: 11px; color: #ef4444;
  }
</style>
