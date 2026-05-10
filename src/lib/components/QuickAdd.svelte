<script lang="ts">
  import { showQuickAdd, addToast, items, projects } from "../stores/app";
  import { createItem, listProjects } from "../api";
  import { onMount } from "svelte";
  import type { Priority, ItemType } from "../types";

  let inputValue = "";
  let inputEl: HTMLInputElement;
  let projectList: { id: string; name: string }[] = [];

  onMount(async () => {
    setTimeout(() => inputEl?.focus(), 50);
    try {
      const p = await listProjects();
      projectList = p.map((x) => ({ id: x.id, name: x.name }));
    } catch {}
  });

  const examples = [
    { tag: "follow-up", text: "Follow up with Infra tomorrow about Kafka provisioning" },
    { tag: "waiting", text: "Waiting on Security for firewall approval" },
    { tag: "issue", text: "Bug: login page crashes on Safari" },
    { tag: "note", text: "Decision: adopted design system v3" },
  ];

  function parseInput(text: string): { type: ItemType; priority: Priority; title: string; projectId?: string } {
    let type: ItemType = "issue";
    let priority: Priority = "p2";
    const lower = text.toLowerCase();
    if (lower.startsWith("follow up") || lower.startsWith("followup") || lower.includes("follow-up")) type = "followup";
    else if (lower.startsWith("waiting") || lower.includes("waiting on")) type = "waiting";
    else if (lower.startsWith("note") || lower.startsWith("decision")) type = "note";
    else if (lower.startsWith("blocked") || lower.includes("blocker")) { type = "issue"; priority = "p0"; }
    else if (lower.includes("bug") || lower.includes("crash")) { type = "issue"; priority = "p1"; }

    // Try to match a project
    let projectId: string | undefined;
    for (const p of projectList) {
      if (lower.includes(p.name.toLowerCase())) {
        projectId = p.id;
        break;
      }
    }
    return { type, priority, title: text, projectId };
  }

  async function handleAdd() {
    if (!inputValue.trim()) return;
    const parsed = parseInput(inputValue.trim());
    const item = await createItem({
      title: parsed.title,
      type: parsed.type,
      priority: parsed.priority,
      project_id: parsed.projectId,
      status: parsed.type === "waiting" ? "waiting" : "open",
    });
    items.update((list) => [item, ...list]);
    addToast(`Added: "${inputValue.slice(0, 45)}${inputValue.length > 45 ? "…" : ""}"`, "✓");
    inputValue = "";
    showQuickAdd.set(false);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") handleAdd();
    if (e.key === "Escape") showQuickAdd.set(false);
  }

  function fillExample(text: string) {
    inputValue = text;
    inputEl?.focus();
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) showQuickAdd.set(false);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="overlay" on:click={handleOverlayClick} role="dialog">
  <div class="modal qa-modal">
    <div class="qa-input-row">
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" style="color:var(--orange);flex-shrink:0">
        <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      <input
        bind:this={inputEl}
        bind:value={inputValue}
        class="qa-inp"
        placeholder="What needs tracking? Natural language OK."
        on:keydown={handleKeydown}
      />
      <button class="btn-icon close-btn" on:click={() => showQuickAdd.set(false)}>
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>
    <div class="qa-examples">
      {#each examples as ex}
        <button class="qa-ex" on:click={() => fillExample(ex.text)}>
          <span class="qa-ex-tag">{ex.tag}</span>
          {ex.text}
        </button>
      {/each}
    </div>
    <div class="qa-footer">
      <span class="qa-hint">Press <kbd>Enter</kbd> to add · <kbd>Esc</kbd> to close</span>
      <button class="btn btn-primary" style="font-size:11px;padding:4px 10px" on:click={handleAdd}>Add</button>
    </div>
  </div>
</div>

<style>
  .qa-modal { width: 520px; }
  .qa-input-row {
    display: flex; align-items: center; gap: 10px;
    padding: 14px 16px; border-block-end: 1px solid var(--border);
  }
  .qa-inp { flex: 1; font-size: 14px; color: var(--fg); background: none; border: none; outline: none; }
  .qa-inp::placeholder { color: var(--fg-4); }
  .close-btn { width: 26px; height: 26px; }
  .qa-examples { padding: 8px 16px; display: flex; flex-direction: column; gap: 2px; }
  .qa-ex {
    display: flex; align-items: center; gap: 8px; padding: 6px 8px;
    border-radius: 6px; cursor: pointer; transition: background 100ms;
    font-size: 11px; color: var(--fg-2); background: none; border: none; text-align: left; width: 100%;
  }
  .qa-ex:hover { background: var(--surface-2); }
  .qa-ex-tag {
    font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em;
    padding: 1px 5px; border-radius: 4px; flex-shrink: 0;
    background: var(--surface-3); color: var(--fg-4);
  }
  .qa-footer {
    padding: 8px 16px; border-block-start: 1px solid var(--border);
    display: flex; justify-content: space-between; align-items: center;
  }
  .qa-hint { font-size: 10px; color: var(--fg-4); }
  .qa-hint kbd { background: var(--surface-2); border: 1px solid var(--border); border-radius: 3px; padding: 1px 4px; font-size: 9px; }
</style>
