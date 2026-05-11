<script lang="ts">
  import { onMount } from "svelte";
  import { items, members, pendingMeTasks, activeView, activeProjectId, addToast } from "../stores/app";
  import { listItems, listMembers, listPendingMeTasks, updateItem, updateTask } from "../api";
  import type { Item, Task, Status } from "../types";
  import IssueModal from "../components/IssueModal.svelte";

  let loading = true;
  let editItem: Item | null = null;
  let showModal = false;

  onMount(async () => {
    loading = true;
    try { const it = await listItems({}); items.set(it); } catch {}
    try { const m = await listMembers(); members.set(m); } catch {}
    try { const t = await listPendingMeTasks(); pendingMeTasks.set(t); } catch {}
    loading = false;
  });

  // Issues & followups pending on me (reactive to $items store)
  $: pendingItems = $items
    .filter(i => i.status === "pending_me")
    .sort((a, b) => {
      const po: Record<string, number> = { p0: 0, p1: 1, p2: 2, p3: 3 };
      return (po[a.priority] ?? 9) - (po[b.priority] ?? 9);
    });

  $: issuesPending   = pendingItems.filter(i => i.type === "issue");
  $: followupsPending = pendingItems.filter(i => i.type === "followup");

  function totalCount(): number {
    return pendingItems.length + $pendingMeTasks.length;
  }

  function fmtDate(d: string) {
    return new Date(d).toLocaleDateString("en-US", { month: "short", day: "numeric" });
  }
  function daysSince(d: string) {
    return Math.floor((Date.now() - new Date(d).getTime()) / 86_400_000);
  }

  // Un-pin an item (set back to 'open')
  async function unpinItem(item: Item) {
    const next: Status = "open";
    await updateItem({ id: item.id, status: next });
    items.update(list => list.map(i => i.id === item.id ? { ...i, status: next } : i));
    addToast(`Unpinned: "${item.title.slice(0, 40)}"`, "⬡");
  }

  // Un-pin a task (set back to 'todo')
  async function unpinTask(task: Task) {
    await updateTask({ id: task.id, status: "todo" });
    pendingMeTasks.update(list => list.filter(t => t.id !== task.id));
    addToast(`Unpinned task: "${task.title.slice(0, 40)}"`, "⬡");
  }

  // Mark item done
  async function doneItem(item: Item) {
    await updateItem({ id: item.id, status: "done" });
    items.update(list => list.map(i => i.id === item.id ? { ...i, status: "done" as Status } : i));
    addToast(`Resolved: "${item.title.slice(0, 40)}"`, "✓");
  }

  // Mark task done
  async function doneTask(task: Task) {
    await updateTask({ id: task.id, status: "done" });
    pendingMeTasks.update(list => list.filter(t => t.id !== task.id));
    addToast(`Done: "${task.title.slice(0, 40)}"`, "✓");
  }

  function openEdit(item: Item) { editItem = item; showModal = true; }
  function closeModal() { showModal = false; editItem = null; }

  function goToProject(task: Task) {
    activeProjectId.set(task.project_id);
    activeView.set("project_detail");
  }
</script>

<!-- Header -->
<div class="page-header pom-header-spacing">
  <div class="pom-header-left">
    <div class="pom-icon-wrap">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/>
        <circle cx="12" cy="7" r="4"/>
        <path d="M16 3.5C17.5 4.5 18 6 18 7"/>
      </svg>
    </div>
    <div>
      <div class="page-title">Pending on Me</div>
      <div class="page-sub">
        {#if loading}Loading…{:else}
          {totalCount()} item{totalCount() !== 1 ? "s" : ""} waiting for your action
        {/if}
      </div>
    </div>
  </div>
</div>

{#if loading}
  <div class="pom-loading">Loading…</div>
{:else if totalCount() === 0}
  <div class="pom-empty">
    <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="color:var(--fg-4)">
      <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/>
      <circle cx="12" cy="7" r="4"/>
      <polyline points="16 11 18 13 22 9" style="stroke:#34d399"/>
    </svg>
    <div class="pom-empty-title">All clear!</div>
    <div class="pom-empty-sub">Nothing is pending on you right now.</div>
  </div>
{:else}
  <!-- Issues Pending -->
  {#if issuesPending.length > 0}
    <div class="pom-section">
      <div class="pom-slabel-row">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
        <span class="pom-slabel">Issues</span>
        <span class="pom-scount">{issuesPending.length}</span>
      </div>
      <div class="pom-list">
        {#each issuesPending as item (item.id)}
          <div class="pom-item" on:click={() => openEdit(item)}>
            <span class="pom-prio-dot {item.priority}"></span>
            <div class="pom-item-body">
              <div class="pom-item-title">{item.title}</div>
              <div class="pom-item-meta">
                {#if item.project_name}<span class="meta-project">{item.project_name}</span><span class="meta-sep">·</span>{/if}
                {#if item.assignee}<span class="meta-text">{item.assignee}</span><span class="meta-sep">·</span>{/if}
                <span class="meta-age">{daysSince(item.updated_at) < 1 ? 'today' : daysSince(item.updated_at) + 'd ago'}</span>
              </div>
            </div>
            <span class="pom-prio-lbl {item.priority}">{item.priority.toUpperCase()}</span>
            <div class="pom-item-actions" on:click|stopPropagation>
              <button class="pom-act done" on:click={() => doneItem(item)} title="Mark done">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
                Done
              </button>
              <button class="pom-act unpin" on:click={() => unpinItem(item)} title="Unpin">Unpin</button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Follow-ups Pending -->
  {#if followupsPending.length > 0}
    <div class="pom-section">
      <div class="pom-slabel-row">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8h1a4 4 0 010 8h-1"/><path d="M2 8h16v9a4 4 0 01-4 4H6a4 4 0 01-4-4V8z"/><line x1="6" y1="1" x2="6" y2="4"/><line x1="10" y1="1" x2="10" y2="4"/><line x1="14" y1="1" x2="14" y2="4"/></svg>
        <span class="pom-slabel">Follow-ups</span>
        <span class="pom-scount">{followupsPending.length}</span>
      </div>
      <div class="pom-list">
        {#each followupsPending as item (item.id)}
          <div class="pom-item" on:click={() => openEdit(item)}>
            <span class="pom-prio-dot {item.priority}"></span>
            <div class="pom-item-body">
              <div class="pom-item-title">{item.title}</div>
              <div class="pom-item-meta">
                {#if item.project_name}<span class="meta-project">{item.project_name}</span><span class="meta-sep">·</span>{/if}
                {#if item.assignee}<span class="meta-text">{item.assignee}</span><span class="meta-sep">·</span>{/if}
                <span class="meta-age">{daysSince(item.updated_at) < 1 ? 'today' : daysSince(item.updated_at) + 'd ago'}</span>
              </div>
            </div>
            <span class="pom-prio-lbl {item.priority}">{item.priority.toUpperCase()}</span>
            <div class="pom-item-actions" on:click|stopPropagation>
              <button class="pom-act done" on:click={() => doneItem(item)} title="Mark done">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
                Done
              </button>
              <button class="pom-act unpin" on:click={() => unpinItem(item)} title="Unpin">Unpin</button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Tasks Pending -->
  {#if $pendingMeTasks.length > 0}
    <div class="pom-section">
      <div class="pom-slabel-row">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="5" width="18" height="14" rx="2"/><polyline points="3 10 21 10"/><line x1="8" y1="15" x2="10" y2="15"/><line x1="13" y1="15" x2="16" y2="15"/></svg>
        <span class="pom-slabel">Tasks</span>
        <span class="pom-scount">{$pendingMeTasks.length}</span>
      </div>
      <div class="pom-list">
        {#each $pendingMeTasks as task (task.id)}
          <div class="pom-item" on:click={() => goToProject(task)}>
            <span class="pom-type-dot"></span>
            <div class="pom-item-body">
              <div class="pom-item-title">{task.title}</div>
              <div class="pom-item-meta">
                {#if task.project_name}<span class="meta-project meta-link">{task.project_name}</span><span class="meta-sep">·</span>{/if}
                {#if task.assignee_name}<span class="meta-text">{task.assignee_name}</span><span class="meta-sep">·</span>{/if}
                <span class="meta-age">{daysSince(task.updated_at) < 1 ? 'today' : daysSince(task.updated_at) + 'd ago'}</span>
              </div>
            </div>
            <div class="pom-item-actions" on:click|stopPropagation>
              <button class="pom-act done" on:click={() => doneTask(task)} title="Mark done">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
                Done
              </button>
              <button class="pom-act unpin" on:click={() => unpinTask(task)} title="Unpin">Unpin</button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
{/if}

{#if showModal}
  <IssueModal item={editItem} on:close={closeModal} />
{/if}

<style>
  /* ── Header ── */
  .pom-header-spacing {
    margin-bottom: 28px;
  }
  .pom-header-left {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .pom-icon-wrap {
    width: 40px; height: 40px;
    border-radius: var(--r-lg);
    background: rgba(99, 102, 241, 0.1);
    border: 1px solid rgba(99, 102, 241, 0.2);
    display: flex; align-items: center; justify-content: center;
    color: #818cf8; flex-shrink: 0;
  }
  /* Extra gap between header and sections */
  :global(.page-header) + .pom-section,
  :global(.page-header) + * + .pom-section {
    margin-top: 0;
  }

  /* ── Loading / Empty ── */
  .pom-loading {
    padding: 60px; text-align: center; color: var(--fg-4); font-size: 13px;
  }
  .pom-empty {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; gap: 12px; padding: 100px 40px; text-align: center;
  }
  .pom-empty-title { font-size: 15px; font-weight: 700; color: var(--fg); }
  .pom-empty-sub { font-size: 12px; color: var(--fg-4); }

  /* ── Section ── */
  .pom-section { margin-bottom: 32px; }

  .pom-slabel-row {
    display: flex; align-items: center; gap: 8px;
    margin-bottom: 12px;
    color: var(--fg-4);
  }
  .pom-slabel {
    font-size: 10px; font-weight: 800; text-transform: uppercase;
    letter-spacing: 0.08em; color: var(--fg-3); flex: 1;
  }
  .pom-scount {
    font-size: 10px; font-weight: 700;
    padding: 1px 8px; border-radius: 99px;
    background: rgba(99, 102, 241, 0.12); color: #818cf8;
  }

  /* ── List container ── */
  .pom-list {
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    overflow: hidden;
  }

  /* ── Individual item ── */
  .pom-item {
    display: flex; align-items: center; gap: 14px;
    padding: 15px 18px;
    border-bottom: 1px solid var(--border);
    cursor: pointer;
    transition: background 80ms;
    position: relative;
  }
  .pom-item::before {
    content: '';
    position: absolute; left: 0; top: 0; bottom: 0; width: 3px;
    background: #6366f1; opacity: 0; transition: opacity 100ms;
    border-radius: 0 2px 2px 0;
  }
  .pom-item:hover { background: var(--surface); }
  .pom-item:hover::before { opacity: 1; }
  .pom-item:last-child { border-bottom: none; }

  /* ── Priority dot ── */
  .pom-prio-dot {
    width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0;
  }
  .pom-prio-dot.p0 { background: #ef4444; box-shadow: 0 0 0 3px rgba(239,68,68,.2); }
  .pom-prio-dot.p1 { background: #f97316; box-shadow: 0 0 0 3px rgba(249,115,22,.18); }
  .pom-prio-dot.p2 { background: #eab308; box-shadow: 0 0 0 3px rgba(234,179,8,.15); }
  .pom-prio-dot.p3 { background: #525252; }
  .pom-type-dot {
    width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0;
    background: #818cf8; box-shadow: 0 0 0 3px rgba(99,102,241,.15);
  }

  /* ── Item body ── */
  .pom-item-body { flex: 1; min-width: 0; }
  .pom-item-title {
    font-size: 13px; font-weight: 560; color: var(--fg);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    margin-bottom: 5px;
  }
  .pom-item-meta {
    display: flex; align-items: center; gap: 5px;
    font-size: 11px; color: var(--fg-4); flex-wrap: wrap;
  }
  .meta-project { color: var(--fg-3); font-weight: 500; }
  .meta-link { color: #818cf8; }
  .meta-sep { color: var(--fg-4); opacity: 0.5; }
  .meta-text { color: var(--fg-3); }
  .meta-age { color: var(--fg-4); }

  /* ── Priority label ── */
  .pom-prio-lbl {
    font-size: 9px; font-weight: 800; padding: 2px 7px; border-radius: 4px;
    flex-shrink: 0; white-space: nowrap; border: 1px solid;
  }
  .pom-prio-lbl.p0 { background: rgba(239,68,68,.1); color: #fca5a5; border-color: rgba(239,68,68,.3); }
  .pom-prio-lbl.p1 { background: rgba(249,115,22,.09); color: #fdba74; border-color: rgba(249,115,22,.3); }
  .pom-prio-lbl.p2 { background: rgba(234,179,8,.08); color: #fef08a; border-color: rgba(234,179,8,.25); }
  .pom-prio-lbl.p3 { background: var(--surface-2); color: var(--fg-3); border-color: var(--border); }

  /* ── Actions ── */
  .pom-item-actions {
    display: flex; align-items: center; gap: 6px; flex-shrink: 0;
    opacity: 0.3; transition: opacity 100ms;
  }
  .pom-item:hover .pom-item-actions { opacity: 1; }
  .pom-act {
    display: flex; align-items: center; gap: 5px;
    padding: 5px 11px; border-radius: var(--r);
    border: 1px solid var(--border); background: var(--surface-2);
    color: var(--fg-3); font-size: 11px; font-weight: 600;
    cursor: pointer; transition: all 100ms; white-space: nowrap;
  }
  .pom-act.done:hover {
    background: rgba(52,211,153,.12); border-color: #34d399; color: #34d399;
  }
  .pom-act.unpin:hover {
    background: rgba(239,68,68,.08); border-color: rgba(239,68,68,.4); color: #f87171;
  }
</style>
