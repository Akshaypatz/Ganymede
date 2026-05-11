<script lang="ts">
  import { onMount } from "svelte";
  import { items, members, projects, followupCategories, addToast } from "../stores/app";
  import { listItems, listMembers, listProjects, createItem, updateItem, deleteItem } from "../api";
  import type { Item, Status } from "../types";
  import ReminderDialog from "../components/ReminderDialog.svelte";

  let showCreate = false;
  let showResolved = false;
  let editingItem: Item | null = null;
  let filterQuery = "";
  let deleteConfirmId: string | null = null;
  let reminderItem: Item | null = null;

  // Create form
  let newTitle = "";
  let newProjectId = "";
  let newPriority = "p2";
  let newAssignee = "";
  let newDueAt = "";
  let newBody = "";
  let newCategory = "";
  let formError = "";

  // Edit form
  let editTitle = "";
  let editProjectId = "";
  let editPriority = "";
  let editStatus = "";
  let editAssignee = "";
  let editDueAt = "";
  let editBody = "";
  let editCategory = "";
  let editError = "";

  onMount(async () => {
    try { const it = await listItems({}); items.set(it); } catch {}
    try { const m = await listMembers(); members.set(m); } catch {}
    try { const p = await listProjects(); projects.set(p); } catch {}
  });

  const PRIO_ORDER: Record<string, number> = { p0: 0, p1: 1, p2: 2, p3: 3 };
  const PRIO_KW    = new Set(["p0","p1","p2","p3"]);
  const STATUS_KW  = new Set(["open","blocked","in_progress","waiting","snoozed"]);

  interface FTok { priorities: string[]; statuses: string[]; texts: string[]; }
  function parseFilter(q: string): FTok {
    const toks = q.trim().toLowerCase().split(/\s+/).filter(Boolean);
    const priorities: string[] = [], statuses: string[] = [], texts: string[] = [];
    for (const t of toks) {
      if (PRIO_KW.has(t))        priorities.push(t);
      else if (STATUS_KW.has(t)) statuses.push(t);
      else                        texts.push(t);
    }
    return { priorities, statuses, texts };
  }

  $: ft = parseFilter(filterQuery);
  $: hasFilter = filterQuery.trim().length > 0;

  $: allFollowups = $items
    .filter(i => i.type === "followup" && i.status !== "done")
    .sort((a, b) => (PRIO_ORDER[a.priority] ?? 9) - (PRIO_ORDER[b.priority] ?? 9));

  $: filteredFollowups = allFollowups.filter(item => {
    if (ft.priorities.length && !ft.priorities.includes(item.priority)) return false;
    if (ft.statuses.length   && !ft.statuses.includes(item.status))     return false;
    if (ft.texts.length) {
      const hay = [item.title, item.assignee ?? "", item.project_name ?? "", item.body ?? ""].join(" ").toLowerCase();
      if (!ft.texts.every(t => hay.includes(t))) return false;
    }
    return true;
  });

  $: pendingMeFollowups     = allFollowups.filter(i => i.status === "pending_me");
  $: activeFiltFollowups    = filteredFollowups.filter(i => i.status !== "pending_me");
  $: highPriorityFollowups  = activeFiltFollowups.filter(i => i.priority === "p0" || i.priority === "p1");
  $: normalFollowups        = activeFiltFollowups.filter(i => i.priority === "p2" || i.priority === "p3");

  $: resolvedFollowups = $items
    .filter(i => i.type === "followup" && i.status === "done")
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
    .slice(0, 30);

  function daysSince(d: string) { return Math.floor((Date.now() - new Date(d).getTime()) / 86_400_000); }
  function fmtDate(d: string) { return new Date(d).toLocaleDateString("en-US", { month: "short", day: "numeric" }); }
  function daysLabel(n: number) { return n < 1 ? "today" : `${n}d`; }
  function isOverdue(item: Item) { return !!item.due_at && new Date(item.due_at) < new Date(); }

  async function handleCreate() {
    formError = "";
    if (!newTitle.trim()) { formError = "Title is required."; return; }
    if (!newProjectId)    { formError = "A project must be selected."; return; }
    const created = await createItem({
      title: newTitle.trim(),
      type: "followup",
      priority: newPriority as any,
      assignee: newAssignee.trim() || undefined,
      due_at: newDueAt || undefined,
      body: newBody.trim() || undefined,
      project_id: newProjectId,
      category: newCategory || undefined,
    });
    items.update(list => [created, ...list]);
    newTitle = ""; newProjectId = ""; newPriority = "p2"; newAssignee = ""; newDueAt = ""; newBody = ""; newCategory = "";
    showCreate = false;
    addToast(`Follow-up created: "${created.title.slice(0, 40)}"`, "✓");
  }

  function openEdit(item: Item) {
    editingItem = item;
    editTitle   = item.title;
    editProjectId = item.project_id || "";
    editPriority  = item.priority;
    editStatus    = item.status;
    editAssignee  = item.assignee || "";
    editDueAt     = item.due_at || "";
    editBody      = item.body || "";
    editCategory  = item.category || "";
    editError     = "";
  }

  async function saveEdit() {
    editError = "";
    if (!editTitle.trim())  { editError = "Title is required."; return; }
    if (!editProjectId)     { editError = "A project must be selected."; return; }
    if (!editingItem) return;
    const updated = await updateItem({
      id: editingItem.id,
      title: editTitle.trim(),
      project_id: editProjectId,
      priority: editPriority as any,
      status: editStatus as any,
      assignee: editAssignee.trim() || undefined,
      due_at: editDueAt || undefined,
      body: editBody.trim() || undefined,
      category: editCategory || undefined,
    });
    items.update(list => list.map(i => i.id === updated.id ? { ...updated, tags: i.tags, project_name: i.project_name } : i));
    try { const it = await listItems({}); items.set(it); } catch {}
    editingItem = null;
    addToast(`Updated: "${editTitle.slice(0, 40)}"`, "✓");
  }

  async function markDone(item: Item) {
    await updateItem({ id: item.id, status: "done" });
    items.update(list => list.map(i => i.id === item.id ? { ...i, status: "done" as const } : i));
    addToast(`Resolved: "${item.title.slice(0, 40)}"`, "✓");
  }
  async function togglePendingMe(item: Item) {
    const next: Status = item.status === "pending_me" ? "open" : "pending_me";
    await updateItem({ id: item.id, status: next });
    items.update(list => list.map(i => i.id === item.id ? { ...i, status: next } : i));
    addToast(next === "pending_me" ? `Pinned to Pending on Me` : `Unpinned from Pending on Me`, "⬡");
  }

  async function reopen(item: Item) {
    await updateItem({ id: item.id, status: "open" as any });
    items.update(list => list.map(i => i.id === item.id ? { ...i, status: "open" as any } : i));
    addToast(`Reopened: "${item.title.slice(0, 40)}"`, "↩");
  }

  async function handleDelete(e: MouseEvent, item: Item) {
    e.stopPropagation();
    if (deleteConfirmId !== item.id) { deleteConfirmId = item.id; return; }
    await deleteItem(item.id);
    items.update(list => list.filter(i => i.id !== item.id));
    deleteConfirmId = null;
    addToast(`Deleted follow-up`, "✗");
  }
</script>

<div class="page-header">
  <div class="page-title">Follow-ups</div>
  <div class="page-sub">
    {allFollowups.length} active
    {#if hasFilter}· <strong>{filteredFollowups.length}</strong> shown{/if}
  </div>
</div>

<div class="toolbar">
  <div class="filter-bar" class:active={hasFilter}>
    <svg class="search-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
    </svg>
    <input
      class="filter-input"
      bind:value={filterQuery}
      placeholder="Filter by priority (P0 P1), status (blocked), project, person…"
    />
    {#if hasFilter}
      <button class="filter-clear" on:click={() => { filterQuery = ""; deleteConfirmId = null; }}>✕</button>
    {/if}
  </div>
  <button class="btn btn-primary" on:click={() => { showCreate = !showCreate; formError = ""; }}>
    {showCreate ? "Cancel" : "+ New Follow-up"}
  </button>
</div>

{#if hasFilter && (ft.priorities.length || ft.statuses.length || ft.texts.length)}
  <div class="token-row">
    {#each ft.priorities as p}<span class="token-pill tok-prio">{p.toUpperCase()}</span>{/each}
    {#each ft.statuses as s}<span class="token-pill tok-status">{s.replace("_"," ")}</span>{/each}
    {#each ft.texts as t}<span class="token-pill tok-text">"{t}"</span>{/each}
  </div>
{/if}

{#if showCreate}
  <div class="create-panel">
    <div class="create-panel-title">New Follow-up</div>
    {#if formError}<div class="form-error">{formError}</div>{/if}
    <div class="create-grid">
      <div class="cg-cell cg-full">
        <label class="field-label">Title <span class="req">*</span></label>
        <input bind:value={newTitle} class="field-input" placeholder="What needs follow-up…"
          on:keydown={(e) => e.key === 'Enter' && handleCreate()} />
      </div>
      <div class="cg-cell">
        <label class="field-label">Project <span class="req">*</span></label>
        <select bind:value={newProjectId} class="field-select" class:field-error={formError && !newProjectId}>
          <option value="">— Select project —</option>
          {#each $projects as p}
            <option value={p.id}>{p.name}</option>
          {/each}
        </select>
      </div>
      <div class="cg-cell">
        <label class="field-label">Priority</label>
        <div class="pill-row">
          {#each [["p0","P0"],["p1","P1"],["p2","P2"],["p3","P3"]] as [v,l]}
            <button class="pill-btn" class:active={newPriority===v} on:click={() => newPriority=v}>{l}</button>
          {/each}
        </div>
      </div>
      <div class="cg-cell">
        <label class="field-label">Assigned to</label>
        <select bind:value={newAssignee} class="field-select">
          <option value="">Unassigned</option>
          {#each $members as m}
            <option value={m.name}>{m.name}</option>
          {/each}
        </select>
      </div>
      <div class="cg-cell">
        <label class="field-label">Due date</label>
        <input bind:value={newDueAt} type="date" class="field-input" />
      </div>
      <div class="cg-cell cg-full">
        <label class="field-label">Notes</label>
        <textarea bind:value={newBody} class="field-textarea" rows="2" placeholder="Context, links, who to contact…"></textarea>
      </div>
      {#if $followupCategories.length > 0}
      <div class="cg-cell">
        <label class="field-label">Category</label>
        <div class="pill-row">
          <button class="pill-btn" class:active={newCategory === ""} on:click={() => newCategory = ""}>None</button>
          {#each $followupCategories as cat}
            <button
              class="pill-btn cat-pill"
              class:active={newCategory === cat.value}
              style={newCategory === cat.value ? `border-color:${cat.color};color:${cat.color};background:${cat.color}18` : ""}
              on:click={() => newCategory = newCategory === cat.value ? "" : cat.value}
            >{cat.label}</button>
          {/each}
        </div>
      </div>
      {/if}
    </div>
    <div class="create-actions">
      <button class="btn btn-ghost" on:click={() => { showCreate = false; formError = ""; }}>Cancel</button>
      <button class="btn btn-primary" on:click={handleCreate}>Create Follow-up</button>
    </div>
  </div>
{/if}

{#if highPriorityFollowups.length > 0}
<div class="section-row high-section">
  <span class="section-dot"></span>
  <span class="section-lbl">Critical &amp; High Priority</span>
  <span class="section-count">{highPriorityFollowups.length}</span>
</div>
<div class="fu-table">
  <div class="table-head">
    <span>Priority</span>
    <span class="col-title">Follow-up</span>
    <span>Project</span>
    <span>Status</span>
    <span>Assigned</span>
    <span>Due</span>
    <span>Age</span>
    <span></span>
  </div>
  {#each highPriorityFollowups as item (item.id)}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <div
      class="table-row hp-row"
      class:overdue={isOverdue(item)}
      class:confirm-pending={deleteConfirmId === item.id}
      on:click={() => openEdit(item)}
      role="row" tabindex="0"
      on:keydown={(e) => e.key === 'Enter' && openEdit(item)}
    >
      <span class="prio-badge {item.priority}">{item.priority.toUpperCase()}</span>
      <span class="col-title row-title">
        {item.title}
        {#if item.category}<span class="cat-tag" style="border-color:{$followupCategories.find(c=>c.value===item.category)?.color||'var(--border)'};color:{$followupCategories.find(c=>c.value===item.category)?.color||'var(--fg-3)'}">{$followupCategories.find(c=>c.value===item.category)?.label||item.category}</span>{/if}
      </span>
      <span class="row-proj" class:missing={!item.project_name}>{item.project_name || "No project"}</span>
      <span class="status-pill {item.status}">{item.status.replace("_"," ")}</span>
      <span class="row-meta">{item.assignee || "—"}</span>
      <span class="row-meta" class:overdue-txt={isOverdue(item)}>
        {item.due_at ? fmtDate(item.due_at) : "—"}
        {#if isOverdue(item)}<span class="overdue-badge">Overdue</span>{/if}
      </span>
      <span class="row-age">{daysLabel(daysSince(item.created_at))}</span>
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <span class="row-actions" on:click|stopPropagation role="group">
        <button class="remind-btn" on:click|stopPropagation={() => reminderItem = item} title="Set reminder"><svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 01-3.46 0"/></svg></button>
        <button class="pin-me-btn" class:active={item.status === 'pending_me'} on:click|stopPropagation={() => togglePendingMe(item)} title={item.status === 'pending_me' ? 'Unpin from Pending on Me' : 'Mark as Pending on Me'}>
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
        </button>
        <button class="resolve-btn" on:click|stopPropagation={() => markDone(item)} title="Resolve">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
        </button>
        {#if deleteConfirmId === item.id}
          <button class="del-confirm" on:click={(e) => handleDelete(e, item)}>Delete?</button>
          <button class="del-cancel" on:click|stopPropagation={() => deleteConfirmId = null}>✕</button>
        {:else}
          <button class="del-btn" on:click={(e) => handleDelete(e, item)} title="Delete">
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6"/><path d="M10 11v6M14 11v6"/></svg>
          </button>
        {/if}
      </span>
    </div>
  {/each}
</div>
{/if}

{#if normalFollowups.length > 0 || highPriorityFollowups.length > 0}
<div class="section-row normal-section">
  <span class="section-lbl">Standard Priority</span>
  {#if normalFollowups.length > 0}<span class="section-count">{normalFollowups.length}</span>{/if}
</div>
{/if}

<div class="fu-table">
  <div class="table-head">
    <span>Priority</span>
    <span class="col-title">Follow-up</span>
    <span>Project</span>
    <span>Status</span>
    <span>Assigned</span>
    <span>Due</span>
    <span>Age</span>
    <span></span>
  </div>

  {#each normalFollowups as item (item.id)}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <div
      class="table-row"
      class:overdue={isOverdue(item)}
      class:confirm-pending={deleteConfirmId === item.id}
      on:click={() => openEdit(item)}
      role="row"
      tabindex="0"
      on:keydown={(e) => e.key === 'Enter' && openEdit(item)}
    >
      <span class="prio-badge {item.priority}">{item.priority.toUpperCase()}</span>
      <span class="col-title row-title">
        {item.title}
        {#if item.category}<span class="cat-tag" style="border-color:{$followupCategories.find(c=>c.value===item.category)?.color||'var(--border)'};color:{$followupCategories.find(c=>c.value===item.category)?.color||'var(--fg-3)'}">{$followupCategories.find(c=>c.value===item.category)?.label||item.category}</span>{/if}
      </span>
      <span class="row-proj" class:missing={!item.project_name}>{item.project_name || "No project"}</span>
      <span class="status-pill {item.status}">{item.status.replace("_"," ")}</span>
      <span class="row-meta">{item.assignee || "—"}</span>
      <span class="row-meta" class:overdue-txt={isOverdue(item)}>
        {item.due_at ? fmtDate(item.due_at) : "—"}
        {#if isOverdue(item)}<span class="overdue-badge">Overdue</span>{/if}
      </span>
      <span class="row-age">{daysLabel(daysSince(item.created_at))}</span>
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <span class="row-actions" on:click|stopPropagation role="group">
        <button class="remind-btn" on:click|stopPropagation={() => reminderItem = item} title="Set reminder"><svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 01-3.46 0"/></svg></button>
        <button class="pin-me-btn" class:active={item.status === 'pending_me'} on:click|stopPropagation={() => togglePendingMe(item)} title={item.status === 'pending_me' ? 'Unpin from Pending on Me' : 'Mark as Pending on Me'}>
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
        </button>
        <button class="resolve-btn" on:click|stopPropagation={() => markDone(item)} title="Resolve">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
        </button>
        {#if deleteConfirmId === item.id}
          <button class="del-confirm" on:click={(e) => handleDelete(e, item)}>Delete?</button>
          <button class="del-cancel" on:click|stopPropagation={() => deleteConfirmId = null}>✕</button>
        {:else}
          <button class="del-btn" on:click={(e) => handleDelete(e, item)} title="Delete">
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6"/><path d="M10 11v6M14 11v6"/></svg>
          </button>
        {/if}
      </span>
    </div>
  {:else}
    <div class="table-empty">
      {#if hasFilter}No follow-ups match your filter{:else}{highPriorityFollowups.length === 0 ? "No active follow-ups" : "No standard-priority follow-ups"}{/if}
    </div>
  {/each}
</div>

{#if pendingMeFollowups.length > 0}
<div class="section-row pending-section">
  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" style="color:#6366f1;flex-shrink:0"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
  <span class="section-lbl pending-lbl">Pending on Me</span>
  <span class="section-count pending-count">{pendingMeFollowups.length}</span>
</div>
<div class="followup-table">
  <div class="table-head">
    <span>Priority</span>
    <span class="col-title">Follow-up</span>
    <span>Project</span>
    <span>Category</span>
    <span>Assignee</span>
    <span>Due</span>
    <span>Age</span>
    <span></span>
  </div>
  {#each pendingMeFollowups as item (item.id)}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <div
      class="table-row pending-row"
      class:confirm-pending={deleteConfirmId === item.id}
      on:click={() => openEdit(item)}
      role="row" tabindex="0"
      on:keydown={(e) => e.key === 'Enter' && openEdit(item)}
    >
      <span class="prio-badge {item.priority}">{item.priority.toUpperCase()}</span>
      <span class="col-title row-title">{item.title}</span>
      <span class="row-proj" class:missing={!item.project_name}>{item.project_name || "No project"}</span>
      <span class="row-meta">{item.category || "—"}</span>
      <span class="row-meta">{item.assignee || "—"}</span>
      <span class="row-meta" class:overdue={isOverdue(item)}>{item.due_at ? fmtDate(item.due_at) : "—"}</span>
      <span class="row-age" class:old={daysSince(item.created_at) > 7}>{daysLabel(daysSince(item.created_at))}</span>
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <span class="row-actions" on:click|stopPropagation role="group">
        <button class="remind-btn" on:click={(e) => openReminder(e, item)} title="Set reminder"><svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 01-3.46 0"/></svg></button>
        <button class="pin-me-btn active" on:click|stopPropagation={() => togglePendingMe(item)} title="Unpin from Pending on Me">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
        </button>
        <button class="resolve-btn" on:click|stopPropagation={() => markDone(item)} title="Resolve">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
        </button>
        {#if deleteConfirmId === item.id}
          <button class="del-confirm" on:click={(e) => handleDelete(e, item)}>Delete?</button>
          <button class="del-cancel" on:click|stopPropagation={() => deleteConfirmId = null}>✕</button>
        {:else}
          <button class="del-btn" on:click={(e) => handleDelete(e, item)} title="Delete">
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6"/><path d="M10 11v6M14 11v6"/></svg>
          </button>
        {/if}
      </span>
    </div>
  {/each}
</div>
{/if}

{#if resolvedFollowups.length > 0}
  <div class="resolved-section">
    <button class="resolved-header" on:click={() => (showResolved = !showResolved)}>
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
        style="transform: rotate({showResolved ? '0deg' : '-90deg'}); transition: transform 150ms">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
      <span>Resolved</span>
      <span class="resolved-count">{resolvedFollowups.length}</span>
    </button>
    {#if showResolved}
      <div class="resolved-list">
        {#each resolvedFollowups as item}
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <div class="resolved-row" on:click={() => openEdit(item)} role="button" tabindex="0" on:keydown={(e) => e.key === 'Enter' && openEdit(item)}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" style="color:var(--green);flex-shrink:0"><polyline points="20 6 9 17 4 12"/></svg>
            <span class="prio-badge {item.priority} sm">{item.priority.toUpperCase()}</span>
            <span class="resolved-title">{item.title}</span>
            <span class="resolved-proj">{item.project_name || ""}</span>
            <span class="resolved-age">{daysLabel(daysSince(item.updated_at))} ago</span>
            <button class="reopen-btn" on:click|stopPropagation={() => reopen(item)}>↩ Reopen</button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<svelte:window on:keydown={(e) => { if (e.key === 'Escape' && editingItem) { e.preventDefault(); editingItem = null; } }} />

<!-- Edit modal -->
{#if editingItem}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <div class="overlay" on:click|self={() => editingItem = null} role="dialog">
    <div class="modal edit-modal">
      <div class="modal-header">
        <span class="modal-title">Edit Follow-up</span>
        <button class="btn-icon" on:click={() => editingItem = null}>✕</button>
      </div>
      <div class="modal-body">
        {#if editError}<div class="form-error">{editError}</div>{/if}
        <div class="field-col">
          <label class="field-label">Title <span class="req">*</span></label>
          <input bind:value={editTitle} class="field-input" />
        </div>
        <div class="field-col">
          <label class="field-label">Project <span class="req">*</span></label>
          <select bind:value={editProjectId} class="field-select" class:field-error={editError && !editProjectId}>
            <option value="">— Select project —</option>
            {#each $projects as p}
              <option value={p.id}>{p.name}</option>
            {/each}
          </select>
        </div>
        <div class="field-row">
          <div class="field-col">
            <label class="field-label">Priority</label>
            <div class="pill-row">
              {#each [["p0","P0"],["p1","P1"],["p2","P2"],["p3","P3"]] as [v,l]}
                <button class="pill-btn" class:active={editPriority===v} on:click={() => editPriority=v}>{l}</button>
              {/each}
            </div>
          </div>
          <div class="field-col">
            <label class="field-label">Status</label>
            <div class="pill-row">
              {#each [["open","Open"],["in_progress","In Progress"],["blocked","Blocked"],["waiting","Waiting"]] as [v,l]}
                <button class="pill-btn" class:active={editStatus===v} on:click={() => editStatus=v}>{l}</button>
              {/each}
            </div>
          </div>
        </div>
        <div class="field-row">
          <div class="field-col">
            <label class="field-label">Assigned to</label>
            <select bind:value={editAssignee} class="field-select">
              <option value="">Unassigned</option>
              {#each $members as m}
                <option value={m.name}>{m.name}</option>
              {/each}
            </select>
          </div>
          <div class="field-col">
            <label class="field-label">Due date</label>
            <input bind:value={editDueAt} type="date" class="field-input" />
          </div>
        </div>
        <div class="field-col">
          <label class="field-label">Notes</label>
          <textarea bind:value={editBody} class="field-textarea" rows="3" placeholder="Context, links…"></textarea>
        </div>
        {#if $followupCategories.length > 0}
        <div class="field-col">
          <label class="field-label">Category</label>
          <div class="pill-row">
            <button class="pill-btn" class:active={editCategory === ""} on:click={() => editCategory = ""}>None</button>
            {#each $followupCategories as cat}
              <button
                class="pill-btn cat-pill"
                class:active={editCategory === cat.value}
                style={editCategory === cat.value ? `border-color:${cat.color};color:${cat.color};background:${cat.color}18` : ""}
                on:click={() => editCategory = editCategory === cat.value ? "" : cat.value}
              >{cat.label}</button>
            {/each}
          </div>
        </div>
        {/if}
      </div>
      <div class="modal-footer">
        <button class="btn btn-danger-ghost" on:click={async (e) => { if (editingItem) { await handleDelete(e, editingItem); editingItem = null; } }}>Delete</button>
        <button class="btn btn-ghost remind-footer-btn" on:click={() => { const it = editingItem; editingItem = null; reminderItem = it; }} title="Set reminder"><svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:-1px;margin-right:4px;"><path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 01-3.46 0"/></svg>Remind</button>
        <div style="flex:1"></div>
        <button class="btn btn-ghost" on:click={() => editingItem = null}>Cancel</button>
        <button class="btn btn-primary" on:click={saveEdit}>Save</button>
      </div>
    </div>
  </div>
{/if}

{#if reminderItem}
  <ReminderDialog
    itemId={reminderItem.id}
    itemTitle={reminderItem.title}
    on:close={() => reminderItem = null}
  />
{/if}

<style>
  .page-header { margin-bottom: 18px; }
  .page-title { font-size: 17px; font-weight: 750; letter-spacing: -0.03em; }
  .page-sub { font-size: 12px; color: var(--fg-3); margin-top: 3px; }

  .toolbar { display: flex; align-items: center; gap: 8px; margin-bottom: 10px; }
  .filter-bar {
    display: flex; align-items: center; gap: 6px; flex: 1;
    background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--r-lg);
    padding: 0 10px; transition: border-color .15s;
  }
  .filter-bar.active, .filter-bar:focus-within { border-color: var(--border-2); }
  .filter-bar:focus-within { border-color: var(--orange); }
  .search-icon { color: var(--fg-4); flex-shrink: 0; }
  .filter-input { flex: 1; background: none; border: none; outline: none; font-size: 12px; color: var(--fg); padding: 8px 0; }
  .filter-input::placeholder { color: var(--fg-4); }
  .filter-clear { background: none; border: none; color: var(--fg-4); cursor: pointer; font-size: 11px; }
  .filter-clear:hover { color: var(--fg); }

  /* Section rows */
  .section-row { display: flex; align-items: center; gap: 7px; margin: 14px 0 6px; }
  .section-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--p0); flex-shrink: 0; }
  .section-lbl { font-size: 10px; font-weight: 800; text-transform: uppercase; letter-spacing: 0.06em; color: var(--fg-3); }
  .high-section .section-lbl { color: var(--p0); }
  .section-count { font-size: 10px; font-weight: 700; padding: 1px 6px; border-radius: 99px; }
  .high-section .section-count { background: rgba(239,68,68,.12); color: var(--p0); }
  .normal-section .section-count { background: var(--surface-2); color: var(--fg-4); }
  .pending-lbl { color: #6366f1; }
  .pending-count { background: rgba(99,102,241,.12); color: #6366f1; }

  /* HP row accent */
  .hp-row, .pending-row { position: relative; }
  .hp-row::before { content: ''; position: absolute; left: 0; top: 0; bottom: 0; width: 3px; background: var(--p0); }
  .pending-row::before { content: ''; position: absolute; left: 0; top: 0; bottom: 0; width: 3px; background: #6366f1; }
  .fu-table { margin-top: 0; }

  /* Category tag inline */
  .cat-tag {
    display: inline-block; font-size: 9px; font-weight: 700; padding: 1px 5px;
    border-radius: 3px; border: 1px solid; margin-left: 5px; vertical-align: middle; white-space: nowrap;
  }

  /* Remind button */
  .remind-btn {
    width: 22px; height: 22px; flex-shrink: 0; display: flex; align-items: center; justify-content: center;
    background: none; border: 1px solid var(--border); border-radius: 5px; font-size: 11px; cursor: pointer;
    opacity: 0; transition: all 100ms;
  }
  .table-row:hover .remind-btn, .hp-row:hover .remind-btn { opacity: 1; }
  .remind-btn:hover { background: rgba(249,115,22,.1); border-color: rgba(249,115,22,.4); }
  .remind-footer-btn { font-size: 11px; }

  .token-row { display: flex; gap: 5px; flex-wrap: wrap; margin-bottom: 10px; }
  .token-pill { font-size: 10px; font-weight: 700; padding: 2px 9px; border-radius: 99px; border: 1px solid; }
  .tok-prio   { background: rgba(239,68,68,.08); color: var(--p0); border-color: rgba(239,68,68,.28); }
  .tok-status { background: rgba(99,102,241,.08); color: #818cf8; border-color: rgba(99,102,241,.28); }
  .tok-text   { background: var(--surface-2); color: var(--fg-3); border-color: var(--border); font-style: italic; }

  /* ── Create panel ── */
  .create-panel {
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg);
    padding: 16px 18px; margin-bottom: 14px;
  }
  .create-panel-title { font-size: 13px; font-weight: 700; margin-bottom: 12px; }
  .create-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .cg-cell { display: flex; flex-direction: column; gap: 4px; }
  .cg-full { grid-column: 1 / -1; }
  .create-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 14px; }

  .form-error {
    padding: 6px 10px; background: rgba(239,68,68,.08); border: 1px solid rgba(239,68,68,.25);
    border-radius: var(--r); font-size: 11px; color: var(--p0); margin-bottom: 8px;
  }
  .req { color: var(--p0); }

  /* ── Table ── */
  .fu-table { border: 1px solid rgba(249,115,22,.14); border-radius: var(--r-lg); overflow: hidden; }

  .table-head {
    display: grid; grid-template-columns: 52px 1fr 140px 100px 100px 110px 50px 90px;
    background: rgba(249,115,22,.05); padding: 7px 12px; gap: 10px;
    font-size: 10px; font-weight: 700; color: var(--fg-3); text-transform: uppercase; letter-spacing: 0.04em;
    border-bottom: 1px solid rgba(249,115,22,.12);
  }

  .table-row {
    display: grid; grid-template-columns: 52px 1fr 140px 100px 100px 110px 50px 90px;
    align-items: center; padding: 9px 12px; gap: 10px;
    border-bottom: 1px solid var(--border); font-size: 12px; cursor: pointer;
    transition: background 100ms;
  }
  .table-row:last-child { border-bottom: none; }
  .table-row:hover { background: rgba(249,115,22,.04); }
  .table-row.overdue { border-left: 3px solid var(--p0); }
  .table-row.confirm-pending { background: rgba(239,68,68,.04); }

  .col-title { min-width: 0; }
  .row-title { font-weight: 560; color: var(--fg); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .row-proj { font-size: 11px; color: var(--fg-3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .row-proj.missing { color: var(--fg-4); font-style: italic; }
  .row-meta { font-size: 11px; color: var(--fg-3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; display: flex; align-items: center; gap: 5px; }
  .overdue-txt { color: var(--p0); font-weight: 600; }
  .overdue-badge { font-size: 9px; font-weight: 700; padding: 1px 5px; border-radius: 99px; background: rgba(239,68,68,.12); color: var(--p0); flex-shrink: 0; }
  .row-age { font-size: 11px; color: var(--fg-4); }

  .prio-badge { font-size: 9px; font-weight: 800; padding: 2px 6px; border-radius: 4px; flex-shrink: 0; white-space: nowrap; }
  .prio-badge.p0 { background: rgba(239,68,68,.12); color: var(--p0); }
  .prio-badge.p1 { background: rgba(249,115,22,.12); color: var(--p1); }
  .prio-badge.p2 { background: rgba(234,179,8,.12); color: var(--p2); }
  .prio-badge.p3 { background: rgba(115,115,115,.12); color: var(--fg-3); }
  .prio-badge.sm { font-size: 8px; padding: 1px 5px; }

  .status-pill { font-size: 9px; font-weight: 700; padding: 2px 7px; border-radius: 99px; border: 1px solid transparent; white-space: nowrap; }
  .status-pill.open        { background: var(--surface-2); color: var(--fg-3); border-color: var(--border); }
  .status-pill.in_progress { background: rgba(99,102,241,.1); color: #818cf8; border-color: rgba(99,102,241,.25); }
  .status-pill.blocked     { background: rgba(239,68,68,.1); color: var(--p0); border-color: rgba(239,68,68,.25); }
  .status-pill.waiting     { background: rgba(245,158,11,.1); color: var(--p1); border-color: rgba(245,158,11,.25); }
  .status-pill.snoozed     { background: var(--surface-2); color: var(--fg-4); border-color: var(--border); }
  .status-pill.pending_me  { background: rgba(99,102,241,.1); color: #6366f1; border-color: rgba(99,102,241,.25); }

  .row-actions { display: flex; align-items: center; gap: 4px; justify-content: flex-end; }
  .resolve-btn {
    width: 22px; height: 22px; flex-shrink: 0; display: flex; align-items: center; justify-content: center;
    background: none; border: 1px solid var(--border); border-radius: 5px; color: var(--fg-4); cursor: pointer; transition: all 100ms;
  }
  .resolve-btn:hover { background: var(--green-bg); border-color: var(--green); color: var(--green); }
  .pin-me-btn {
    width: 22px; height: 22px; flex-shrink: 0; display: flex; align-items: center; justify-content: center;
    background: none; border: 1px solid var(--border); border-radius: 5px; color: var(--fg-4); cursor: pointer;
    transition: all 100ms; opacity: 0;
  }
  .table-row:hover .pin-me-btn, .hp-row:hover .pin-me-btn { opacity: 1; }
  .pin-me-btn:hover { background: rgba(99,102,241,.1); border-color: rgba(99,102,241,.4); color: #6366f1; }
  .pin-me-btn.active { opacity: 1; background: rgba(99,102,241,.15); border-color: rgba(99,102,241,.5); color: #6366f1; }
  .del-btn {
    width: 22px; height: 22px; flex-shrink: 0; display: flex; align-items: center; justify-content: center;
    background: none; border: 1px solid var(--border); border-radius: 5px; color: var(--fg-4); cursor: pointer;
    transition: all 100ms; opacity: 0;
  }
  .table-row:hover .del-btn { opacity: 1; }
  .del-btn:hover { background: rgba(239,68,68,.1); border-color: rgba(239,68,68,.4); color: var(--p0); }
  .del-confirm { padding: 2px 8px; font-size: 10px; font-weight: 700; border-radius: 4px; cursor: pointer; background: rgba(239,68,68,.12); border: 1px solid rgba(239,68,68,.4); color: var(--p0); }
  .del-cancel  { width: 20px; height: 20px; display: flex; align-items: center; justify-content: center; background: none; border: none; color: var(--fg-4); cursor: pointer; font-size: 11px; }

  .table-empty { padding: 32px; text-align: center; font-size: 12px; color: var(--fg-4); }

  /* Resolved */
  .resolved-section { margin-top: 24px; border-top: 1px solid var(--border); padding-top: 14px; }
  .resolved-header { display: flex; align-items: center; gap: 8px; width: 100%; background: none; border: none; cursor: pointer; padding: 4px 0; font-size: 11px; font-weight: 700; color: var(--fg-3); text-align: left; letter-spacing: 0.04em; text-transform: uppercase; }
  .resolved-header:hover { color: var(--fg); }
  .resolved-count { font-size: 10px; background: var(--surface-2); padding: 1px 7px; border-radius: 99px; color: var(--fg-4); }
  .resolved-list { margin-top: 6px; border: 1px solid var(--border); border-radius: var(--r-lg); overflow: hidden; }
  .resolved-row { display: flex; align-items: center; gap: 10px; padding: 8px 12px; border-bottom: 1px solid var(--border); font-size: 11px; cursor: pointer; transition: background 100ms; }
  .resolved-row:last-child { border-bottom: none; }
  .resolved-row:hover { background: var(--surface-2); }
  .resolved-title { flex: 1; color: var(--fg-3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; }
  .resolved-proj  { font-size: 10px; color: var(--fg-4); min-width: 90px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .resolved-age   { font-size: 10px; color: var(--fg-4); flex-shrink: 0; }
  .reopen-btn { opacity: 0; padding: 2px 8px; font-size: 10px; border-radius: 4px; cursor: pointer; background: var(--surface-2); border: 1px solid var(--border); color: var(--fg-3); transition: all 100ms; flex-shrink: 0; }
  .resolved-row:hover .reopen-btn { opacity: 1; }
  .reopen-btn:hover { background: var(--orange-bg); color: var(--orange); border-color: var(--orange); }

  /* Edit modal */
  .edit-modal { width: 520px; max-height: 88vh; overflow-y: auto; }
  .modal-header { display: flex; align-items: center; justify-content: space-between; padding: 14px 18px; border-bottom: 1px solid var(--border); }
  .modal-title { font-size: 13px; font-weight: 700; }
  .modal-body { padding: 16px 18px; display: flex; flex-direction: column; gap: 12px; }
  .modal-footer { padding: 12px 18px; border-top: 1px solid var(--border); display: flex; align-items: center; gap: 8px; }
  .field-label { font-size: 10px; font-weight: 700; color: var(--fg-4); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px; }
  .field-col { display: flex; flex-direction: column; }
  .field-row { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
  .field-input { padding: 7px 10px; background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg); }
  .field-input:focus { border-color: var(--orange); outline: none; }
  .field-select { padding: 7px 10px; background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg); }
  .field-select:focus { border-color: var(--orange); outline: none; }
  .field-select.field-error { border-color: var(--p0); }
  .field-textarea { padding: 7px 10px; background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg); resize: vertical; font-family: inherit; }
  .field-textarea:focus { border-color: var(--orange); outline: none; }
  .pill-row { display: flex; gap: 4px; }
  .pill-btn { padding: 4px 10px; border-radius: 99px; font-size: 11px; font-weight: 600; border: 1px solid var(--border); background: var(--surface-2); color: var(--fg-3); cursor: pointer; transition: all 100ms; }
  .pill-btn.active { background: var(--orange-bg); border-color: var(--orange); color: var(--orange); }
  .btn-danger-ghost { padding: 6px 12px; border-radius: var(--r); border: 1px solid rgba(239,68,68,.3); background: none; color: var(--p0); font-size: 12px; font-weight: 600; cursor: pointer; transition: all 100ms; }
  .btn-danger-ghost:hover { background: rgba(239,68,68,.1); border-color: rgba(239,68,68,.5); }
</style>
