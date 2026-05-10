<script lang="ts">
  import { onMount } from "svelte";
  import { items, members, addToast } from "../stores/app";
  import { listItems, listMembers, createItem, updateItem, deleteItem } from "../api";
  import type { Item } from "../types";

  const statuses = [
    { key: "open",        label: "Scoping",    color: "#737373", bg: "rgba(115,115,115,.1)" },
    { key: "in_progress", label: "Active",      color: "#3b82f6", bg: "rgba(59,130,246,.1)" },
    { key: "snoozed",     label: "Paused",      color: "#eab308", bg: "rgba(234,179,8,.1)" },
    { key: "done",        label: "Completed",   color: "#22c55e", bg: "rgba(34,197,94,.1)" },
  ];

  let filterStatus = "";
  let showCreate = false;
  let newTitle = "";
  let newDesc = "";
  let newOwner = "";
  let newStatus = "open";
  let newTarget = "";
  let editingItem: Item | null = null;
  let editTitle = "";
  let editDesc = "";
  let editOwner = "";
  let editStatus = "";
  let editTarget = "";

  onMount(async () => {
    try { const it = await listItems({}); items.set(it); } catch {}
    try { const m = await listMembers(); members.set(m); } catch {}
  });

  $: allInitiatives = $items.filter((i) => i.type === "initiative");
  $: initiatives = filterStatus
    ? allInitiatives.filter((i) => i.status === filterStatus)
    : allInitiatives.filter((i) => i.status !== "done");
  $: completed = allInitiatives.filter((i) => i.status === "done")
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime());

  let showCompleted = false;

  function statusDef(key: string) {
    return statuses.find((s) => s.key === key) || statuses[0];
  }

  function daysSince(dateStr: string) {
    const diff = Math.floor((Date.now() - new Date(dateStr).getTime()) / 86400000);
    return diff < 1 ? "Today" : `${diff}d ago`;
  }

  function formatDate(dateStr: string) {
    return new Date(dateStr).toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
  }

  async function handleCreate() {
    if (!newTitle.trim()) return;
    const item = await createItem({
      title: newTitle.trim(),
      body: newDesc.trim(),
      type: "initiative",
      status: newStatus as any,
      priority: "p2",
      assignee: newOwner || undefined,
      due_at: newTarget || undefined,
    });
    items.update((list) => [item, ...list]);
    newTitle = ""; newDesc = ""; newOwner = ""; newStatus = "open"; newTarget = "";
    showCreate = false;
    addToast(`Initiative added: "${item.title.slice(0, 40)}"`, "✓");
  }

  function openEdit(item: Item) {
    editingItem = item;
    editTitle = item.title;
    editDesc = item.body;
    editOwner = item.assignee;
    editStatus = item.status;
    editTarget = item.due_at || "";
  }

  async function saveEdit() {
    if (!editingItem) return;
    const updated = await updateItem({
      id: editingItem.id,
      title: editTitle,
      body: editDesc,
      assignee: editOwner,
      status: editStatus as any,
      due_at: editTarget || undefined,
    });
    items.update((list) => list.map((i) => i.id === updated.id ? { ...updated, tags: i.tags, project_name: i.project_name } : i));
    editingItem = null;
    addToast(`Updated: "${editTitle.slice(0, 40)}"`, "✓");
  }

  async function removeInitiative(item: Item) {
    await deleteItem(item.id);
    items.update((list) => list.filter((i) => i.id !== item.id));
    editingItem = null;
    addToast(`Removed: "${item.title.slice(0, 40)}"`, "✗");
  }
</script>

<div class="page-header">
  <div class="page-hdr-row">
    <div>
      <div class="page-title">Initiatives</div>
      <div class="page-sub">Org-level strategic initiatives — beyond individual projects and issues</div>
    </div>
    <button class="btn btn-primary" on:click={() => (showCreate = !showCreate)}>+ New Initiative</button>
  </div>
</div>

<!-- Status filter pills -->
<div class="filter-bar">
  <button class="filter-pill" class:active={filterStatus === ""} on:click={() => filterStatus = ""}>Active</button>
  {#each statuses.filter(s => s.key !== "done") as s}
    <button
      class="filter-pill"
      class:active={filterStatus === s.key}
      style={filterStatus === s.key ? `background:${s.bg};color:${s.color};border-color:${s.color}55` : ""}
      on:click={() => filterStatus = filterStatus === s.key ? "" : s.key}
    >{s.label}</button>
  {/each}
  <div class="filter-spacer"></div>
  <span class="filter-count">{initiatives.length} initiative{initiatives.length !== 1 ? "s" : ""}</span>
</div>

<!-- Create form -->
{#if showCreate}
  <div class="create-card">
    <input bind:value={newTitle} placeholder="Initiative name…" class="create-input" autofocus
      on:keydown={(e) => e.key === "Enter" && !e.shiftKey && handleCreate()} />
    <textarea bind:value={newDesc} placeholder="What is this initiative about? Goals, context, why it matters…" class="create-textarea" rows="3"></textarea>
    <div class="create-row">
      <select bind:value={newStatus} class="create-select">
        {#each statuses as s}<option value={s.key}>{s.label}</option>{/each}
      </select>
      <input bind:value={newOwner} placeholder="Owner" class="create-input short" />
      <input bind:value={newTarget} type="date" class="create-input short" title="Target date" />
      <button class="btn btn-primary" on:click={handleCreate}>Add</button>
      <button class="btn btn-ghost" on:click={() => (showCreate = false)}>Cancel</button>
    </div>
  </div>
{/if}

<!-- Initiative cards -->
<div class="initiative-list">
  {#each initiatives as item (item.id)}
    {@const s = statusDef(item.status)}
    <div class="initiative-card" on:click={() => openEdit(item)} role="button" tabindex="0"
      on:keydown={(e) => e.key === "Enter" && openEdit(item)}>
      <div class="init-left">
        <div class="init-status-bar" style="background:{s.color}"></div>
      </div>
      <div class="init-body">
        <div class="init-header">
          <div class="init-title">{item.title}</div>
          <span class="status-chip" style="background:{s.bg};color:{s.color}">{s.label}</span>
        </div>
        {#if item.body}
          <div class="init-desc">{item.body.slice(0, 140)}{item.body.length > 140 ? "…" : ""}</div>
        {/if}
        <div class="init-meta">
          {#if item.assignee}
            <span class="init-owner">
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
              {item.assignee}
            </span>
          {/if}
          {#if item.due_at}
            <span class="init-target" class:overdue={new Date(item.due_at) < new Date()}>
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
              {formatDate(item.due_at)}
            </span>
          {/if}
          <span class="init-age">{daysSince(item.created_at)}</span>
        </div>
      </div>
    </div>
  {:else}
    <div class="empty-state">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
      <p>No active initiatives. Add strategic org-level goals here.</p>
    </div>
  {/each}
</div>

<!-- Completed section -->
{#if completed.length > 0}
  <div class="completed-section">
    <button class="completed-hdr" on:click={() => (showCompleted = !showCompleted)}>
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
        style="transform: rotate({showCompleted ? '0deg' : '-90deg'}); transition: transform 150ms">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
      <span>Completed</span>
      <span class="completed-count">{completed.length}</span>
    </button>
    {#if showCompleted}
      <div class="completed-list">
        {#each completed as item}
          <div class="completed-row" on:click={() => openEdit(item)} role="button" tabindex="0" on:keydown={(e) => e.key === "Enter" && openEdit(item)}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" style="color:var(--green);flex-shrink:0"><polyline points="20 6 9 17 4 12"/></svg>
            <span class="completed-title">{item.title}</span>
            {#if item.assignee}<span class="completed-owner">{item.assignee}</span>{/if}
            <span class="completed-age">{daysSince(item.updated_at)}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<!-- Edit modal -->
{#if editingItem}
  <div class="overlay" on:click|self={() => (editingItem = null)} role="dialog">
    <div class="modal init-modal">
      <div class="modal-header">
        <span class="modal-title">Initiative</span>
        <div class="modal-hdr-actions">
          <button class="btn-sm danger" on:click={() => editingItem && removeInitiative(editingItem)} title="Delete">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-2 14H7L5 6"/></svg>
          </button>
          <button class="btn-icon" on:click={() => (editingItem = null)}>✕</button>
        </div>
      </div>
      <div class="modal-body">
        <label class="field-label">Title</label>
        <input bind:value={editTitle} class="field-input" />
        <label class="field-label">Description</label>
        <textarea bind:value={editDesc} class="field-textarea" rows="5" placeholder="Goals, success criteria, context…"></textarea>
        <div class="field-row">
          <div class="field-col">
            <label class="field-label">Status</label>
            <select bind:value={editStatus} class="field-select">
              {#each statuses as s}<option value={s.key}>{s.label}</option>{/each}
            </select>
          </div>
          <div class="field-col">
            <label class="field-label">Owner</label>
            <input bind:value={editOwner} class="field-input" placeholder="Name" />
          </div>
        </div>
        <label class="field-label">Target Date</label>
        <input bind:value={editTarget} type="date" class="field-input" />
      </div>
      <div class="modal-footer">
        <button class="btn btn-ghost" on:click={() => (editingItem = null)}>Cancel</button>
        <button class="btn btn-primary" on:click={saveEdit}>Save</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .page-header { margin-bottom: 14px; }
  .page-hdr-row { display: flex; align-items: flex-start; justify-content: space-between; }
  .page-title { font-size: 17px; font-weight: 750; letter-spacing: -0.03em; }
  .page-sub { font-size: 12px; color: var(--fg-3); margin-top: 3px; }

  .filter-bar { display: flex; align-items: center; gap: 6px; margin-bottom: 16px; flex-wrap: wrap; }
  .filter-pill {
    padding: 4px 12px; border-radius: 99px; font-size: 11px; font-weight: 500;
    border: 1px solid var(--border); background: var(--surface-2); color: var(--fg-3);
    cursor: pointer; transition: all 120ms;
  }
  .filter-pill:hover { border-color: var(--border-2); color: var(--fg); }
  .filter-pill.active { background: var(--surface-3); border-color: var(--border-2); color: var(--fg); }
  .filter-spacer { flex: 1; }
  .filter-count { font-size: 11px; color: var(--fg-4); }

  .create-card {
    background: var(--surface); border: 1px solid var(--orange); border-radius: var(--r-lg);
    padding: 14px 16px; margin-bottom: 16px; display: flex; flex-direction: column; gap: 8px;
  }
  .create-input {
    padding: 7px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r); font-size: 13px; color: var(--fg);
  }
  .create-input:focus { border-color: var(--orange); outline: none; }
  .create-textarea {
    padding: 7px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg);
    resize: vertical; font-family: inherit;
  }
  .create-textarea:focus { border-color: var(--orange); outline: none; }
  .create-row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .create-select { flex-shrink: 0; }
  .create-input.short { width: 130px; flex: none; font-size: 12px; }

  .initiative-list { display: flex; flex-direction: column; gap: 8px; }
  .initiative-card {
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg);
    display: flex; overflow: hidden; cursor: pointer; transition: all 130ms;
  }
  .initiative-card:hover { border-color: var(--border-2); box-shadow: var(--shadow); transform: translateY(-1px); }
  .init-left { width: 4px; flex-shrink: 0; }
  .init-status-bar { width: 100%; height: 100%; }
  .init-body { flex: 1; padding: 12px 14px; }
  .init-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 10px; margin-bottom: 6px; }
  .init-title { font-size: 13px; font-weight: 600; color: var(--fg); flex: 1; line-height: 1.4; }
  .status-chip {
    font-size: 9px; font-weight: 700; padding: 2px 8px; border-radius: 99px;
    text-transform: uppercase; letter-spacing: 0.06em; flex-shrink: 0;
  }
  .init-desc { font-size: 11px; color: var(--fg-3); line-height: 1.5; margin-bottom: 8px; }
  .init-meta { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
  .init-owner, .init-target, .init-age {
    display: flex; align-items: center; gap: 4px;
    font-size: 10px; color: var(--fg-4);
  }
  .init-target.overdue { color: var(--p0); }

  .empty-state {
    display: flex; flex-direction: column; align-items: center; gap: 10px;
    padding: 40px 0; color: var(--fg-4);
  }
  .empty-state p { font-size: 12px; color: var(--fg-4); }

  .completed-section { margin-top: 24px; }
  .completed-hdr {
    display: flex; align-items: center; gap: 8px; font-size: 11px; font-weight: 600;
    color: var(--fg-3); background: none; border: none; cursor: pointer; padding: 6px 0;
    text-transform: uppercase; letter-spacing: 0.04em;
  }
  .completed-hdr:hover { color: var(--fg); }
  .completed-count { font-size: 10px; background: var(--surface-2); padding: 1px 7px; border-radius: 99px; color: var(--fg-4); }
  .completed-list { margin-top: 6px; display: flex; flex-direction: column; gap: 2px; }
  .completed-row {
    display: flex; align-items: center; gap: 10px; padding: 7px 10px;
    border-radius: var(--r); cursor: pointer; transition: background 100ms; font-size: 11px;
  }
  .completed-row:hover { background: var(--surface-2); }
  .completed-title { flex: 1; color: var(--fg-3); text-decoration: line-through; text-decoration-color: var(--border-2); }
  .completed-owner { font-size: 10px; color: var(--fg-4); }
  .completed-age { font-size: 10px; color: var(--fg-4); white-space: nowrap; flex-shrink: 0; }

  /* Modal */
  .init-modal { width: 520px; max-height: 85vh; overflow-y: auto; }
  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 18px; border-bottom: 1px solid var(--border);
  }
  .modal-hdr-actions { display: flex; align-items: center; gap: 6px; }
  .modal-title { font-size: 13px; font-weight: 700; }
  .modal-body { padding: 16px 18px; display: flex; flex-direction: column; gap: 10px; }
  .modal-footer { padding: 12px 18px; border-top: 1px solid var(--border); display: flex; gap: 8px; justify-content: flex-end; }
  .field-label { font-size: 10px; font-weight: 600; color: var(--fg-3); text-transform: uppercase; letter-spacing: 0.04em; }
  .field-input {
    width: 100%; padding: 7px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg); outline: none;
  }
  .field-input:focus { border-color: var(--orange); }
  .field-select {
    width: 100%; padding: 7px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg);
  }
  .field-textarea {
    width: 100%; padding: 7px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg);
    resize: vertical; font-family: inherit; outline: none;
  }
  .field-textarea:focus { border-color: var(--orange); }
  .field-row { display: flex; gap: 10px; }
  .field-col { flex: 1; display: flex; flex-direction: column; gap: 6px; }
  .btn-sm {
    padding: 4px 8px; border-radius: 5px; font-size: 10px; display: flex; align-items: center; gap: 3px;
    border: 1px solid var(--border); background: none; color: var(--fg-3); cursor: pointer; transition: all 100ms;
  }
  .btn-sm.danger:hover { background: var(--p0-bg); border-color: var(--p0); color: var(--p0); }
</style>
