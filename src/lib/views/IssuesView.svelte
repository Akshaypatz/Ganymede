<script lang="ts">
  import { onMount } from "svelte";
  import { items, members, projects, addToast } from "../stores/app";
  import { listItems, listMembers, listProjects, updateItem, deleteItem } from "../api";
  import type { Item, Status } from "../types";
  import IssueModal from "../components/IssueModal.svelte";

  let showResolved = false;
  let showModal = false;
  let editItem: Item | null = null;
  let filterQuery = "";
  let deleteConfirmId: string | null = null;

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

  $: allIssues = $items
    .filter(i => i.type === "issue" && i.status !== "done")
    .sort((a, b) => (PRIO_ORDER[a.priority] ?? 9) - (PRIO_ORDER[b.priority] ?? 9));

  $: filteredIssues = allIssues.filter(item => {
    if (ft.priorities.length && !ft.priorities.includes(item.priority)) return false;
    if (ft.statuses.length   && !ft.statuses.includes(item.status))     return false;
    if (ft.texts.length) {
      const hay = [item.title, item.assignee ?? "", item.project_name ?? ""].join(" ").toLowerCase();
      if (!ft.texts.every(t => hay.includes(t))) return false;
    }
    return true;
  });

  $: resolvedIssues = $items
    .filter(i => i.type === "issue" && i.status === "done")
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
    .slice(0, 40);

  $: p0Count = allIssues.filter(i => i.priority === "p0").length;
  $: p1Count = allIssues.filter(i => i.priority === "p1").length;
  $: p2Count = allIssues.filter(i => i.priority === "p2").length;
  $: p3Count = allIssues.filter(i => i.priority === "p3").length;

  function daysSince(d: string) { return Math.floor((Date.now() - new Date(d).getTime()) / 86_400_000); }
  function fmtDate(d: string) { return new Date(d).toLocaleDateString("en-US", { month: "short", day: "numeric" }); }
  function daysLabel(n: number) { return n < 1 ? "today" : `${n}d`; }

  async function markDone(item: Item) {
    await updateItem({ id: item.id, status: "done" });
    items.update(list => list.map(i => i.id === item.id ? { ...i, status: "done" as const } : i));
    addToast(`Resolved: "${item.title.slice(0, 40)}"`, "✓");
  }
  async function reopen(item: Item) {
    await updateItem({ id: item.id, status: "open" as Status });
    items.update(list => list.map(i => i.id === item.id ? { ...i, status: "open" as Status } : i));
    addToast(`Reopened: "${item.title.slice(0, 40)}"`, "↩");
  }
  async function handleDelete(e: MouseEvent, item: Item) {
    e.stopPropagation();
    if (deleteConfirmId !== item.id) { deleteConfirmId = item.id; return; }
    await deleteItem(item.id);
    items.update(list => list.filter(i => i.id !== item.id));
    deleteConfirmId = null;
    addToast(`Deleted issue`, "✗");
  }

  function openCreate() { editItem = null; showModal = true; }
  function openEdit(item: Item) { editItem = item; showModal = true; }
  function closeModal() { showModal = false; editItem = null; }
</script>

<div class="page-header">
  <div class="page-title">Issues</div>
  <div class="page-sub">
    {allIssues.length} open
    {#if hasFilter}· <strong>{filteredIssues.length}</strong> shown{/if}
  </div>
</div>

<div class="summary-row">
  <div class="stat-card red"><div class="sc-label">P0 Critical</div><div class="sc-num">{p0Count}</div></div>
  <div class="stat-card orange"><div class="sc-label">P1 High</div><div class="sc-num">{p1Count}</div></div>
  <div class="stat-card yellow"><div class="sc-label">P2 Medium</div><div class="sc-num">{p2Count}</div></div>
  <div class="stat-card gray"><div class="sc-label">P3 Low</div><div class="sc-num">{p3Count}</div></div>
</div>

<div class="toolbar">
  <div class="filter-bar" class:active={hasFilter}>
    <svg class="search-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
    </svg>
    <input
      class="filter-input"
      bind:value={filterQuery}
      placeholder="Filter by priority (p0), status (blocked), project, assignee…"
    />
    {#if hasFilter}
      <button class="filter-clear" on:click={() => { filterQuery = ""; deleteConfirmId = null; }}>✕</button>
    {/if}
  </div>
  <button class="btn btn-primary" on:click={openCreate}>+ New Issue</button>
</div>

{#if hasFilter && (ft.priorities.length || ft.statuses.length || ft.texts.length)}
  <div class="token-row">
    {#each ft.priorities as p}<span class="token-pill tok-prio">{p.toUpperCase()}</span>{/each}
    {#each ft.statuses as s}<span class="token-pill tok-status">{s.replace("_"," ")}</span>{/each}
    {#each ft.texts as t}<span class="token-pill tok-text">"{t}"</span>{/each}
  </div>
{/if}

<div class="issue-table">
  <div class="table-head">
    <span>Priority</span>
    <span class="col-title">Issue</span>
    <span>Project</span>
    <span>Status</span>
    <span>Assignee</span>
    <span>Due</span>
    <span>Age</span>
    <span></span>
  </div>

  {#each filteredIssues as item (item.id)}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <div
      class="table-row"
      class:confirm-pending={deleteConfirmId === item.id}
      on:click={() => openEdit(item)}
      role="row"
      tabindex="0"
      on:keydown={(e) => e.key === 'Enter' && openEdit(item)}
    >
      <span class="prio-badge {item.priority}">{item.priority.toUpperCase()}</span>
      <span class="col-title row-title">{item.title}</span>
      <span class="row-proj" class:missing={!item.project_name}>{item.project_name || "No project"}</span>
      <span class="status-pill {item.status}">{item.status.replace("_"," ")}</span>
      <span class="row-meta">{item.assignee || "—"}</span>
      <span class="row-meta" class:overdue={item.due_at && new Date(item.due_at) < new Date()}>
        {item.due_at ? fmtDate(item.due_at) : "—"}
      </span>
      <span class="row-age" class:old={daysSince(item.created_at) > 7}>{daysLabel(daysSince(item.created_at))}</span>
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <span class="row-actions" on:click|stopPropagation role="group">
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
      {#if hasFilter}No issues match your filter{:else}No open issues — great work!{/if}
    </div>
  {/each}
</div>

{#if resolvedIssues.length > 0}
  <div class="resolved-section">
    <button class="resolved-header" on:click={() => (showResolved = !showResolved)}>
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
        style="transform: rotate({showResolved ? '0deg' : '-90deg'}); transition: transform 150ms">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
      <span>Resolved</span>
      <span class="resolved-count">{resolvedIssues.length}</span>
    </button>
    {#if showResolved}
      <div class="resolved-list">
        {#each resolvedIssues as item}
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

{#if showModal}
  <IssueModal item={editItem} on:close={closeModal} />
{/if}

<style>
  .page-header { margin-bottom: 18px; }
  .page-title { font-size: 17px; font-weight: 750; letter-spacing: -0.03em; }
  .page-sub { font-size: 12px; color: var(--fg-3); margin-top: 3px; }

  .summary-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin-bottom: 14px; }
  .stat-card { background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg); padding: 12px 14px; position: relative; overflow: hidden; }
  .stat-card::after { content: ''; position: absolute; top: 0; left: 0; right: 0; height: 2px; }
  .stat-card.red::after    { background: var(--p0); }
  .stat-card.orange::after { background: var(--orange); }
  .stat-card.yellow::after { background: var(--p2); }
  .stat-card.gray::after   { background: var(--fg-4); }
  .sc-label { font-size: 10px; color: var(--fg-3); font-weight: 500; }
  .sc-num { font-size: 26px; font-weight: 800; letter-spacing: -0.05em; font-variant-numeric: tabular-nums; line-height: 1; }

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

  .token-row { display: flex; gap: 5px; flex-wrap: wrap; margin-bottom: 10px; }
  .token-pill { font-size: 10px; font-weight: 700; padding: 2px 9px; border-radius: 99px; border: 1px solid; }
  .tok-prio   { background: rgba(239,68,68,.08); color: var(--p0); border-color: rgba(239,68,68,.28); }
  .tok-status { background: rgba(99,102,241,.08); color: #818cf8; border-color: rgba(99,102,241,.28); }
  .tok-text   { background: var(--surface-2); color: var(--fg-3); border-color: var(--border); font-style: italic; }

  /* ── Table ── */
  .issue-table { margin-top: 16px; border: 1px solid var(--border); border-radius: var(--r-lg); overflow: hidden; }

  .table-head {
    display: grid; grid-template-columns: 52px 1fr 140px 100px 100px 80px 50px 90px;
    background: var(--surface-2); padding: 7px 12px; gap: 10px;
    font-size: 10px; font-weight: 700; color: var(--fg-4); text-transform: uppercase; letter-spacing: 0.04em;
    border-bottom: 1px solid var(--border);
  }

  .table-row {
    display: grid; grid-template-columns: 52px 1fr 140px 100px 100px 80px 50px 90px;
    align-items: center; padding: 9px 12px; gap: 10px;
    border-bottom: 1px solid var(--border); font-size: 12px; cursor: pointer;
    transition: background 100ms;
  }
  .table-row:last-child { border-bottom: none; }
  .table-row:hover { background: var(--surface-2); }
  .table-row.confirm-pending { background: rgba(239,68,68,.04); }

  .col-title { min-width: 0; }
  .row-title { font-weight: 560; color: var(--fg); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .row-proj { font-size: 11px; color: var(--fg-3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .row-proj.missing { color: var(--fg-4); font-style: italic; }
  .row-meta { font-size: 11px; color: var(--fg-3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .row-meta.overdue { color: var(--p0); font-weight: 600; }
  .row-age { font-size: 11px; color: var(--fg-4); font-variant-numeric: tabular-nums; }
  .row-age.old { color: var(--p0); font-weight: 700; }

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

  .row-actions { display: flex; align-items: center; gap: 4px; justify-content: flex-end; }
  .resolve-btn {
    width: 22px; height: 22px; flex-shrink: 0; display: flex; align-items: center; justify-content: center;
    background: none; border: 1px solid var(--border); border-radius: 5px; color: var(--fg-4); cursor: pointer;
    transition: all 100ms;
  }
  .resolve-btn:hover { background: var(--green-bg); border-color: var(--green); color: var(--green); }
  .del-btn {
    width: 22px; height: 22px; flex-shrink: 0; display: flex; align-items: center; justify-content: center;
    background: none; border: 1px solid var(--border); border-radius: 5px; color: var(--fg-4); cursor: pointer;
    transition: all 100ms; opacity: 0;
  }
  .table-row:hover .del-btn { opacity: 1; }
  .del-btn:hover { background: rgba(239,68,68,.1); border-color: rgba(239,68,68,.4); color: var(--p0); }
  .del-confirm {
    padding: 2px 8px; font-size: 10px; font-weight: 700; border-radius: 4px; cursor: pointer;
    background: rgba(239,68,68,.12); border: 1px solid rgba(239,68,68,.4); color: var(--p0);
  }
  .del-cancel {
    width: 20px; height: 20px; display: flex; align-items: center; justify-content: center;
    background: none; border: none; color: var(--fg-4); cursor: pointer; font-size: 11px;
  }

  .table-empty { padding: 32px; text-align: center; font-size: 12px; color: var(--fg-4); }

  /* Resolved */
  .resolved-section { margin-top: 24px; border-top: 1px solid var(--border); padding-top: 14px; }
  .resolved-header {
    display: flex; align-items: center; gap: 8px; width: 100%;
    background: none; border: none; cursor: pointer; padding: 4px 0;
    font-size: 11px; font-weight: 700; color: var(--fg-3); text-align: left;
    letter-spacing: 0.04em; text-transform: uppercase;
  }
  .resolved-header:hover { color: var(--fg); }
  .resolved-count { font-size: 10px; background: var(--surface-2); padding: 1px 7px; border-radius: 99px; color: var(--fg-4); }
  .resolved-list { margin-top: 6px; border: 1px solid var(--border); border-radius: var(--r-lg); overflow: hidden; }
  .resolved-row {
    display: flex; align-items: center; gap: 10px; padding: 8px 12px;
    border-bottom: 1px solid var(--border); font-size: 11px; cursor: pointer; transition: background 100ms;
  }
  .resolved-row:last-child { border-bottom: none; }
  .resolved-row:hover { background: var(--surface-2); }
  .resolved-title { flex: 1; color: var(--fg-3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; }
  .resolved-proj  { font-size: 10px; color: var(--fg-4); min-width: 90px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .resolved-age   { font-size: 10px; color: var(--fg-4); flex-shrink: 0; }
  .reopen-btn {
    opacity: 0; padding: 2px 8px; font-size: 10px; border-radius: 4px; cursor: pointer;
    background: var(--surface-2); border: 1px solid var(--border); color: var(--fg-3); transition: all 100ms; flex-shrink: 0;
  }
  .resolved-row:hover .reopen-btn { opacity: 1; }
  .reopen-btn:hover { background: var(--orange-bg); color: var(--orange); border-color: var(--orange); }
</style>
