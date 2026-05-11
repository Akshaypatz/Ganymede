<script lang="ts">
  import {
    activeView,
    sidebarCollapsed,
    boards,
    items,
    activeBoardId,
    checkinReport,
    currentUser,
    isLocked,
    pendingMeTasks,
  } from "./stores/app";
  import { listBoards, listItems } from "./api";
  import { onMount } from "svelte";
  import type { ViewId } from "./types";
  import GanymedeMark from "./components/GanymedeMark.svelte";

  let boardList: { id: string; name: string }[] = [];

  onMount(async () => {
    try {
      const b = await listBoards();
      boards.set(b);
      boardList = b.map((x) => ({ id: x.id, name: x.name }));
    } catch {}
    try {
      const it = await listItems();
      items.set(it);
    } catch {}
  });

  function switchView(id: ViewId) {
    activeView.set(id);
  }

  function openBoard(id: string) {
    activeView.set("board");
    activeBoardId.set(id);
  }

  $: issueCount = $items.filter((i) => i.type === "issue" && i.status !== "done").length;
  $: followupCount = $items.filter((i) => i.type === "followup" && i.status !== "done").length;
  $: ideaCount = $items.filter((i) => i.type === "idea" && i.status !== "done").length;
  $: initiativeCount = $items.filter((i) => i.type === "initiative" && i.status !== "done").length;
  $: pendingOnMeCount = $items.filter((i) => i.status === "pending_me").length + $pendingMeTasks.length;
  $: userName = $currentUser?.name ?? "";
  $: initials = userName
    ? userName.trim().split(/\s+/).map((w: string) => w[0]).slice(0, 2).join("").toUpperCase()
    : "?";
</script>

<nav class="sidebar" class:collapsed={$sidebarCollapsed}>
  <div class="sidebar-header">
    <img src="/billdesk-logo.png" class="logo-mark" alt="BillDesk" />
    {#if !$sidebarCollapsed}
      <div class="logo-name">Ganymede <span>Mission Control</span></div>
    {/if}
  </div>

  <div class="sidebar-nav">
    <button
      class="nav-item"
      class:active={$activeView === "dashboard"}
      on:click={() => switchView("dashboard")}
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>
      {#if !$sidebarCollapsed}<span class="nav-label">Dashboard</span>{/if}
    </button>

    {#if !$sidebarCollapsed}<div class="section-label">Triage</div>{/if}
    <button
      class="nav-item"
      class:active={$activeView === "issues"}
      on:click={() => switchView("issues")}
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
      {#if !$sidebarCollapsed}
        <span class="nav-label">Issues</span>
        {#if issueCount > 0}<span class="nav-count urgent">{issueCount}</span>{/if}
      {/if}
    </button>

    <button
      class="nav-item"
      class:active={$activeView === "followups"}
      on:click={() => switchView("followups")}
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8h1a4 4 0 010 8h-1"/><path d="M2 8h16v9a4 4 0 01-4 4H6a4 4 0 01-4-4V8z"/><line x1="6" y1="1" x2="6" y2="4"/><line x1="10" y1="1" x2="10" y2="4"/><line x1="14" y1="1" x2="14" y2="4"/></svg>
      {#if !$sidebarCollapsed}
        <span class="nav-label">Follow-ups</span>
        {#if followupCount > 0}<span class="nav-count">{followupCount}</span>{/if}
      {/if}
    </button>

    <button
      class="nav-item"
      class:active={$activeView === "pending_on_me"}
      on:click={() => switchView("pending_on_me")}
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:{$activeView === 'pending_on_me' ? '#818cf8' : 'currentColor'}"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
      {#if !$sidebarCollapsed}
        <span class="nav-label" style="color:{$activeView === 'pending_on_me' ? '#818cf8' : ''}">Pending on Me</span>
        {#if pendingOnMeCount > 0}<span class="nav-count pom">{pendingOnMeCount}</span>{/if}
      {/if}
    </button>

    {#if !$sidebarCollapsed}<div class="section-label">Plan</div>{/if}
    <button
      class="nav-item"
      class:active={$activeView === "ideas"}
      on:click={() => switchView("ideas")}
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2a7 7 0 017 7c0 2.38-1.19 4.47-3 5.74V17a2 2 0 01-2 2h-4a2 2 0 01-2-2v-2.26C6.19 13.47 5 11.38 5 9a7 7 0 017-7z"/><line x1="9" y1="21" x2="15" y2="21"/></svg>
      {#if !$sidebarCollapsed}
        <span class="nav-label">Ideas</span>
        {#if ideaCount > 0}<span class="nav-count">{ideaCount}</span>{/if}
      {/if}
    </button>

    <button
      class="nav-item"
      class:active={$activeView === "initiatives"}
      on:click={() => switchView("initiatives")}
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
      {#if !$sidebarCollapsed}
        <span class="nav-label">Initiatives</span>
        {#if initiativeCount > 0}<span class="nav-count">{initiativeCount}</span>{/if}
      {/if}
    </button>

    {#if !$sidebarCollapsed}<div class="section-label">Work</div>{/if}
    <button
      class="nav-item"
      class:active={$activeView === "projects"}
      on:click={() => switchView("projects")}
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 3h6a4 4 0 014 4v14a3 3 0 00-3-3H2z"/><path d="M22 3h-6a4 4 0 00-4 4v14a3 3 0 013-3h7z"/></svg>
      {#if !$sidebarCollapsed}<span class="nav-label">Projects</span>{/if}
    </button>

    {#each boardList as board}
      <button
        class="nav-item"
        class:active={$activeView === "board"}
        on:click={() => openBoard(board.id)}
      >
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="5" height="18" rx="1"/><rect x="10" y="3" width="5" height="11" rx="1"/><rect x="17" y="3" width="5" height="15" rx="1"/></svg>
        {#if !$sidebarCollapsed}<span class="nav-label">{board.name}</span>{/if}
      </button>
    {/each}

    {#if !$sidebarCollapsed}<div class="section-label">AI & Settings</div>{/if}
    <button
      class="nav-item"
      class:active={$activeView === 'checkin'}
      on:click={() => switchView('checkin')}
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/><path d="M8 14h.01M12 14h.01M16 14h.01M8 18h.01M12 18h.01"/></svg>
      {#if !$sidebarCollapsed}
        <span class="nav-label">Daily Brief</span>
        {#if ($checkinReport?.total_attention ?? 0) > 0}
          <span class="nav-count urgent">{$checkinReport?.total_attention}</span>
        {/if}
      {/if}
    </button>
    <button
      class="nav-item"
      class:active={$activeView === 'ai'}
      on:click={() => switchView('ai')}
    >
      <GanymedeMark size={13} />
      {#if !$sidebarCollapsed}<span class="nav-label">Ask Ganymede</span>{/if}
    </button>
    <button
      class="nav-item"
      class:active={$activeView === "settings"}
      on:click={() => switchView("settings")}
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
      {#if !$sidebarCollapsed}<span class="nav-label">Settings</span>{/if}
    </button>
  </div>

  <div class="sidebar-footer">
    <div class="user-row">
      <div class="avatar">{initials}</div>
      {#if !$sidebarCollapsed}<span class="user-name">{userName || "Ganymede"}</span>{/if}
      <button class="lock-btn" on:click={() => isLocked.set(true)} title="Lock screen (⌘L)">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0110 0v4"/>
        </svg>
      </button>
    </div>
  </div>
</nav>

<style>
  .sidebar {
    width: var(--sidebar-w);
    flex-shrink: 0;
    background: var(--surface);
    border-inline-end: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    transition: width var(--t);
    z-index: 20;
    overflow: hidden;
  }
  .sidebar.collapsed { width: 48px; }
  .sidebar-header {
    padding: 12px 14px;
    display: flex;
    align-items: center;
    gap: 10px;
    border-block-end: 1px solid var(--border);
    flex-shrink: 0;
    min-height: 52px;
  }
  .logo-mark {
    width: 30px; height: 30px; flex-shrink: 0; object-fit: contain;
  }
  .nav-bd-icon { width: 14px; height: 14px; object-fit: contain; flex-shrink: 0; opacity: 0.85; }
  .logo-name {
    display: flex; flex-direction: column; line-height: 1.1;
    font-size: 13px; font-weight: 700; letter-spacing: -0.01em; color: var(--fg);
  }
  .logo-name span { font-size: 9px; color: var(--fg-3); font-weight: 500; letter-spacing: 0.08em; text-transform: uppercase; }
  .sidebar-nav { flex: 1; overflow-y: auto; padding: 8px 6px; }
  .section-label {
    font-size: 9px; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase;
    color: var(--fg-4); padding: 10px 10px 4px;
  }
  .nav-item {
    display: flex; align-items: center; gap: 8px;
    padding: 7px 10px; border-radius: var(--r);
    color: var(--fg-3); font-size: 12px; font-weight: 450;
    cursor: pointer; transition: all 120ms; user-select: none;
    width: 100%; text-align: left;
  }
  .nav-item:hover { background: var(--surface-2); color: var(--fg); }
  .nav-item.active { background: var(--orange-bg); color: var(--orange); font-weight: 600; }
  .nav-item :global(svg) { flex-shrink: 0; opacity: .7; }
  .nav-item.active :global(svg) { opacity: 1; }
  .nav-label { flex: 1; }
  .nav-count {
    font-size: 10px; font-weight: 700; min-width: 18px; height: 16px;
    border-radius: 99px; display: flex; align-items: center; justify-content: center;
    padding: 0 4px; background: var(--orange-bg); color: var(--orange);
  }
  .nav-count.urgent { background: var(--p0-bg); color: var(--p0); }
  .nav-count.pom { background: rgba(99,102,241,.15); color: #818cf8; }
  .sidebar-footer { padding: 8px 6px; border-block-start: 1px solid var(--border); flex-shrink: 0; }
  .user-row {
    display: flex; align-items: center; gap: 8px; padding: 7px 10px;
    border-radius: var(--r); cursor: pointer; transition: background 120ms;
  }
  .user-row:hover { background: var(--surface-2); }
  .avatar {
    width: 26px; height: 26px; border-radius: 50%; flex-shrink: 0;
    background: linear-gradient(135deg, var(--orange-deep), var(--orange-dim));
    display: flex; align-items: center; justify-content: center;
    font-size: 10px; font-weight: 800; color: #fff;
  }
  .user-name { font-size: 12px; font-weight: 500; color: var(--fg-2); flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .lock-btn {
    width: 24px; height: 24px; flex-shrink: 0;
    display: flex; align-items: center; justify-content: center;
    background: none; border: 1px solid transparent; border-radius: 6px;
    color: var(--fg-4); cursor: pointer; transition: all 120ms;
    margin-left: auto; opacity: 0;
  }
  .user-row:hover .lock-btn { opacity: 1; }
  .lock-btn:hover { background: rgba(249,115,22,.1); border-color: rgba(249,115,22,.3); color: var(--orange); }

  .collapsed .nav-item { padding: 9px; justify-content: center; }
  .collapsed .nav-count { display: none; }
  .collapsed .sidebar-header { padding: 12px; justify-content: center; }
</style>
