<script lang="ts">
  import {
    sidebarCollapsed,
    isDark,
    showCommandPalette,
    showQuickAdd,
    checkinReport,
    activeView,
  } from "./stores/app";
  import { setSetting } from "./api";

  function toggleSidebar() {
    sidebarCollapsed.update((v) => !v);
  }

  function toggleTheme() {
    isDark.update((v) => {
      const next = !v;
      document.documentElement.setAttribute(
        "data-theme",
        next ? "dark" : "light"
      );
      setSetting("theme", next ? "dark" : "light").catch(() => {});
      return next;
    });
  }

  $: attentionCount = $checkinReport?.total_attention ?? 0;
</script>

<div class="topbar">
  <button class="btn-icon" on:click={toggleSidebar}>
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <line x1="3" y1="6" x2="21" y2="6"/>
      <line x1="3" y1="12" x2="21" y2="12"/>
      <line x1="3" y1="18" x2="21" y2="18"/>
    </svg>
  </button>

  <button class="search-bar" on:click={() => showCommandPalette.set(true)}>
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--fg-4)">
      <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
    </svg>
    <span>Search or jump to…</span>
    <kbd>⌘K</kbd>
  </button>

  <div class="topbar-right">
    {#if attentionCount > 0}
      <button class="nudge-btn" on:click={() => activeView.set('checkin')} title="{attentionCount} items need attention">
        <span class="nudge-dot"></span>
        <span class="nudge-count">{attentionCount}</span>
      </button>
    {/if}
    <button class="btn btn-ghost" on:click={() => showQuickAdd.set(true)}>
      <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19"/>
        <line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      Add
    </button>

    <button class="btn-icon" on:click={toggleTheme}>
      {#if $isDark}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/></svg>
      {:else}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>
      {/if}
    </button>

    <div class="avatar">AP</div>
  </div>
</div>

<style>
  .topbar {
    height: var(--topbar-h); flex-shrink: 0;
    background: var(--surface);
    border-block-end: 1px solid var(--border);
    display: flex; align-items: center; gap: 8px; padding: 0 14px;
    z-index: 10;
  }
  .search-bar {
    flex: 1; max-width: 340px;
    display: flex; align-items: center; gap: 7px;
    background: var(--surface-2); border: 1px solid var(--border);
    border-radius: var(--r); padding: 6px 10px;
    cursor: pointer; transition: border-color 150ms;
  }
  .search-bar:hover { border-color: var(--border-2); }
  .search-bar span { color: var(--fg-4); font-size: 12px; flex: 1; }
  .search-bar kbd {
    font-size: 10px; color: var(--fg-4); background: var(--surface);
    border: 1px solid var(--border); border-radius: 4px; padding: 1px 5px;
  }
  .topbar-right { margin-inline-start: auto; display: flex; align-items: center; gap: 5px; }
  .nudge-btn {
    display: flex; align-items: center; gap: 5px;
    padding: 4px 10px; border-radius: var(--r); border: 1px solid rgba(249,115,22,.3);
    background: rgba(249,115,22,.08); cursor: pointer; transition: all 120ms;
    color: var(--orange); font-size: 11px; font-weight: 700;
  }
  .nudge-btn:hover { background: rgba(249,115,22,.15); border-color: rgba(249,115,22,.5); }
  .nudge-dot {
    width: 7px; height: 7px; border-radius: 50%; background: var(--orange);
    animation: nudge-pulse 2s ease infinite;
    flex-shrink: 0;
  }
  @keyframes nudge-pulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(249,115,22,.5); }
    50% { box-shadow: 0 0 0 5px rgba(249,115,22,0); }
  }
  .nudge-count { line-height: 1; }
  .avatar {
    width: 26px; height: 26px; border-radius: 50%; flex-shrink: 0;
    background: linear-gradient(135deg, var(--orange-deep), var(--orange-dim));
    display: flex; align-items: center; justify-content: center;
    font-size: 10px; font-weight: 800; color: #fff; cursor: pointer;
  }
</style>
