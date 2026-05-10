<script lang="ts">
  import { showCommandPalette, activeView, projects, boards, activeBoardId, isDark, addToast } from "../stores/app";
  import { onMount } from "svelte";
  import { setSetting } from "../api";
  import type { ViewId } from "../types";

  let query = "";
  let inputEl: HTMLInputElement;
  let selectedIndex = 0;

  interface PaletteItem {
    icon: string;
    label: string;
    group: string;
    action: () => void;
  }

  const viewActions: PaletteItem[] = [
    { icon: "⌂", label: "Dashboard", group: "Views", action: () => go("dashboard") },
    { icon: "⚠", label: "Issues", group: "Views", action: () => go("issues") },
    { icon: "◫", label: "Projects", group: "Views", action: () => go("projects") },
  ];

  const miscActions: PaletteItem[] = [
    { icon: "◑", label: "Toggle theme", group: "Actions", action: () => toggleTheme() },
  ];

  function go(view: ViewId) {
    activeView.set(view);
    showCommandPalette.set(false);
  }

  function toggleTheme() {
    isDark.update((v) => {
      const next = !v;
      document.documentElement.setAttribute("data-theme", next ? "dark" : "light");
      setSetting("theme", next ? "dark" : "light").catch(() => {});
      return next;
    });
    showCommandPalette.set(false);
  }

  $: boardActions = $boards.map((b) => ({
    icon: "▤",
    label: b.name,
    group: "Boards",
    action: () => {
      activeBoardId.set(b.id);
      activeView.set("board");
      showCommandPalette.set(false);
    },
  })) as PaletteItem[];

  $: projectActions = $projects.map((p) => ({
    icon: "●",
    label: p.name,
    group: "Projects",
    action: () => {
      addToast(`Opening ${p.name}…`);
      showCommandPalette.set(false);
    },
  })) as PaletteItem[];

  $: allItems = [...viewActions, ...boardActions, ...projectActions, ...miscActions];
  $: filtered = query
    ? allItems.filter((i) => i.label.toLowerCase().includes(query.toLowerCase()))
    : allItems;

  $: if (selectedIndex >= filtered.length) selectedIndex = Math.max(0, filtered.length - 1);

  onMount(() => {
    setTimeout(() => inputEl?.focus(), 50);
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "ArrowDown") { e.preventDefault(); selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1); }
    if (e.key === "ArrowUp") { e.preventDefault(); selectedIndex = Math.max(selectedIndex - 1, 0); }
    if (e.key === "Enter" && filtered[selectedIndex]) { filtered[selectedIndex].action(); }
    if (e.key === "Escape") { showCommandPalette.set(false); }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) showCommandPalette.set(false);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="overlay" on:click={handleOverlayClick} role="dialog">
  <div class="modal palette-modal">
    <div class="pal-input-row">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--fg-4);flex-shrink:0">
        <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      <input
        bind:this={inputEl}
        bind:value={query}
        class="pal-inp"
        placeholder="Jump to view, project, or action…"
        on:keydown={handleKeydown}
      />
    </div>
    <div class="pal-list">
      {#each filtered as item, i}
        {#if i === 0 || filtered[i - 1]?.group !== item.group}
          <div class="pal-group">{item.group}</div>
        {/if}
        <button
          class="pal-item"
          class:selected={i === selectedIndex}
          on:click={item.action}
          on:mouseenter={() => (selectedIndex = i)}
        >
          <div class="pal-icon">{item.icon}</div>
          {item.label}
        </button>
      {:else}
        <div class="pal-empty">No results</div>
      {/each}
    </div>
  </div>
</div>

<style>
  .palette-modal { width: 480px; }
  .pal-input-row {
    display: flex; align-items: center; gap: 9px;
    padding: 12px 14px; border-block-end: 1px solid var(--border);
  }
  .pal-inp { flex: 1; font-size: 13px; color: var(--fg); background: none; border: none; outline: none; }
  .pal-inp::placeholder { color: var(--fg-4); }
  .pal-list { padding: 6px; max-height: 280px; overflow-y: auto; }
  .pal-group {
    font-size: 9px; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase;
    color: var(--fg-4); padding: 6px 8px 2px;
  }
  .pal-item {
    display: flex; align-items: center; gap: 9px; padding: 7px 9px;
    border-radius: 6px; cursor: pointer; transition: background 80ms;
    font-size: 12px; color: var(--fg-2); background: none; border: none; width: 100%; text-align: left;
  }
  .pal-item:hover, .pal-item.selected { background: var(--orange-bg); color: var(--orange); }
  .pal-icon {
    width: 22px; height: 22px; border-radius: 6px; background: var(--surface-2);
    display: flex; align-items: center; justify-content: center; font-size: 11px; flex-shrink: 0;
  }
  .pal-empty { font-size: 12px; color: var(--fg-4); padding: 16px; text-align: center; }
</style>
