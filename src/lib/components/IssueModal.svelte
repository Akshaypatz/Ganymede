<script lang="ts">
  import { items, members, projects, addToast } from "../stores/app";
  import { createItem, updateItem, deleteItem, createMember, listMembers, listItems } from "../api";
  import type { Item, Member, Priority, Status } from "../types";
  import { createEventDispatcher } from "svelte";

  export let item: Item | null = null;
  export let mode: "create" | "edit" = item ? "edit" : "create";

  const dispatch = createEventDispatcher<{ close: void }>();

  let title = item?.title || "";
  let body = item?.body || "";
  let priority: Priority = (item?.priority as Priority) || "p2";
  let status: Status = (item?.status as Status) || "open";
  let assignee = item?.assignee || "";
  let projectId = item?.project_id || "";
  let dueAt = item?.due_at || "";

  let formError = "";
  let deleteConfirm = false;
  let showNewMember = false;
  let newMemberName = "";
  let newMemberEmail = "";

  async function addNewMember() {
    if (!newMemberName.trim()) return;
    try {
      const m = await createMember({ name: newMemberName.trim(), email: newMemberEmail.trim() || undefined });
      members.update((list) => [...list, m]);
      assignee = m.name;
      newMemberName = "";
      newMemberEmail = "";
      showNewMember = false;
      addToast(`Member added: ${m.name}`, "✓");
    } catch (e: any) {
      addToast(`Failed: ${e}`, "✗");
    }
  }

  async function handleSave() {
    formError = "";
    if (!title.trim())  { formError = "Title is required."; return; }
    if (!projectId)     { formError = "A project must be selected before saving."; return; }
    if (mode === "create") {
      const created = await createItem({
        title: title.trim(),
        body: body.trim(),
        type: "issue",
        priority,
        status,
        assignee: assignee || undefined,
        project_id: projectId,
        due_at: dueAt || undefined,
      });
      items.update((list) => [created, ...list]);
      addToast(`Issue created: "${title.slice(0, 40)}"`, "✓");
    } else if (item) {
      const updated = await updateItem({
        id: item.id,
        title: title.trim(),
        body: body.trim(),
        type: "issue",
        priority,
        status,
        assignee: assignee || undefined,
        project_id: projectId,
        due_at: dueAt || undefined,
      });
      items.update((list) => list.map((i) => (i.id === updated.id ? { ...updated, tags: i.tags, project_name: i.project_name } : i)));
      try { const it = await listItems({}); items.set(it); } catch {}
      addToast(`Updated: "${title.slice(0, 40)}"`, "✓");
    }
    dispatch("close");
  }

  async function handleDelete() {
    if (!deleteConfirm) { deleteConfirm = true; return; }
    if (!item) return;
    await deleteItem(item.id);
    items.update(list => list.filter(i => i.id !== item!.id));
    addToast(`Deleted issue`, "✗");
    dispatch("close");
  }

  function close() { dispatch("close"); }
</script>

<div class="overlay" on:click|self={close} role="dialog">
  <div class="modal issue-modal">
    <div class="modal-header">
      <span class="modal-title">{mode === "create" ? "New Issue" : "Edit Issue"}</span>
      <button class="btn-icon" on:click={close}>✕</button>
    </div>

    <div class="modal-body">
      {#if formError}
        <div class="form-error">{formError}</div>
      {/if}

      <!-- Title -->
      <input
        bind:value={title}
        class="title-input"
        placeholder="Issue title…"
        on:keydown={(e) => e.key === "Enter" && handleSave()}
      />

      <!-- Project (required) -->
      <div class="field-col">
        <div class="field-label">Project <span class="req">*</span></div>
        <select bind:value={projectId} class="field-select" class:field-error={formError && !projectId}>
          <option value="">— Select a project —</option>
          {#each $projects as p}
            <option value={p.id}>{p.name}</option>
          {/each}
        </select>
        {#if $projects.length === 0}
          <span class="no-projects-hint">No projects yet — create a project first</span>
        {/if}
      </div>

      <!-- Priority + Status row -->
      <div class="field-row-2">
        <div class="field-col">
          <div class="field-label">Priority</div>
          <div class="pill-group">
            {#each [["p0","P0","var(--p0)","var(--p0-bg)"],["p1","P1","var(--p1)","var(--p1-bg)"],["p2","P2","var(--p2)","var(--p2-bg)"],["p3","P3","var(--fg-4)","var(--surface-2)"]] as [val,lbl,c,bg]}
              <button
                class="pill" class:pill-active={priority === val}
                style={priority === val ? `background:${bg};border-color:${c};color:${c}` : ""}
                on:click={() => priority = val as any}
              >{lbl}</button>
            {/each}
          </div>
        </div>
        <div class="field-col">
          <div class="field-label">Status</div>
          <div class="pill-group">
            {#each [["open","Open"],["in_progress","In Progress"],["blocked","Blocked"]] as [val,lbl]}
              <button
                class="pill" class:pill-active={status === val}
                on:click={() => status = val as any}
              >{lbl}</button>
            {/each}
          </div>
        </div>
      </div>

      <!-- Assignee + Due row -->
      <div class="field-row-2">
        <div class="field-col">
          <div class="field-label">Assignee</div>
          <div class="select-row">
            <select bind:value={assignee} class="field-select">
              <option value="">Unassigned</option>
              {#each $members as m}
                <option value={m.name}>{m.name}</option>
              {/each}
            </select>
            <button class="add-member-btn" on:click={() => (showNewMember = !showNewMember)} title="Add member">+</button>
          </div>
          {#if showNewMember}
            <div class="new-member-row">
              <input bind:value={newMemberName} placeholder="Name" class="field-input" />
              <input bind:value={newMemberEmail} placeholder="Email (opt.)" class="field-input" />
              <button class="btn btn-primary" style="font-size:10px;padding:4px 10px;white-space:nowrap" on:click={addNewMember}>Add</button>
            </div>
          {/if}
        </div>
        <div class="field-col">
          <div class="field-label">Due date</div>
          <input bind:value={dueAt} type="date" class="field-input" />
        </div>
      </div>

      <!-- Description -->
      <div class="field-col">
        <div class="field-label">Notes / Description</div>
        <textarea bind:value={body} class="field-textarea" rows="3" placeholder="Details, links, context…"></textarea>
      </div>
    </div>

    <div class="modal-footer">
      {#if mode === "edit"}
        {#if deleteConfirm}
          <button class="btn btn-danger-confirm" on:click={handleDelete}>Confirm Delete</button>
          <button class="btn btn-ghost" on:click={() => deleteConfirm = false}>Cancel</button>
        {:else}
          <button class="btn btn-danger-ghost" on:click={handleDelete}>Delete</button>
        {/if}
        <div style="flex:1"></div>
      {/if}
      <button class="btn btn-ghost" on:click={close}>Cancel</button>
      <button class="btn btn-primary" on:click={handleSave}>{mode === "create" ? "Create Issue" : "Save Changes"}</button>
    </div>
  </div>
</div>

<style>
  .issue-modal { width: 540px; max-height: 88vh; overflow-y: auto; }
  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 16px 20px; border-bottom: 1px solid var(--border);
  }
  .modal-title { font-size: 14px; font-weight: 700; letter-spacing: -0.02em; }

  .modal-body { padding: 20px; display: flex; flex-direction: column; gap: 16px; }
  .modal-footer { padding: 14px 20px; border-top: 1px solid var(--border); display: flex; align-items: center; gap: 8px; }

  .title-input {
    width: 100%; padding: 10px 12px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r-lg);
    font-size: 14px; font-weight: 600; color: var(--fg); letter-spacing: -0.01em;
    transition: border-color 120ms;
  }
  .title-input::placeholder { color: var(--fg-4); font-weight: 400; }
  .title-input:focus { border-color: var(--orange); outline: none; }

  .field-label { font-size: 10px; font-weight: 700; color: var(--fg-4); text-transform: uppercase; letter-spacing: 0.06em; margin-bottom: 5px; }
  .field-row-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
  .field-col { display: flex; flex-direction: column; }

  /* Pill group */
  .pill-group { display: flex; gap: 4px; flex-wrap: wrap; }
  .pill {
    padding: 4px 11px; border-radius: 99px; font-size: 11px; font-weight: 600; cursor: pointer;
    border: 1px solid var(--border); background: var(--surface-2); color: var(--fg-3);
    transition: all 100ms;
  }
  .pill:hover { border-color: var(--border-2); color: var(--fg); }
  .pill-active { background: var(--orange-bg) !important; border-color: var(--orange) !important; color: var(--orange) !important; }

  /* Override for priority pills */
  .pill[style].pill-active { }

  .select-row { display: flex; gap: 4px; }
  .select-row .field-select { flex: 1; }

  .field-select {
    width: 100%; padding: 7px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg);
  }
  .field-select:focus { border-color: var(--orange); outline: none; }

  .field-input {
    flex: 1; padding: 7px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg);
  }
  .field-input:focus { border-color: var(--orange); outline: none; }

  .field-textarea {
    width: 100%; padding: 8px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r-lg); font-size: 12px; color: var(--fg);
    resize: vertical; font-family: inherit; line-height: 1.55; min-height: 72px;
  }
  .field-textarea:focus { border-color: var(--orange); outline: none; }

  .add-member-btn {
    width: 32px; height: 32px; flex-shrink: 0; display: flex; align-items: center; justify-content: center;
    background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--r);
    color: var(--fg-3); font-size: 16px; cursor: pointer; transition: all 100ms;
  }
  .add-member-btn:hover { background: var(--orange-bg); border-color: var(--orange); color: var(--orange); }

  .new-member-row { display: flex; gap: 5px; margin-top: 6px; }

  .form-error {
    padding: 7px 10px; background: rgba(239,68,68,.08); border: 1px solid rgba(239,68,68,.25);
    border-radius: var(--r); font-size: 11px; color: var(--p0);
  }
  .req { color: var(--p0); }
  .field-error { border-color: var(--p0) !important; }
  .no-projects-hint { font-size: 10px; color: var(--p0); margin-top: 4px; }

  .btn-danger-ghost {
    padding: 6px 12px; border-radius: var(--r); border: 1px solid rgba(239,68,68,.3);
    background: none; color: var(--p0); font-size: 12px; font-weight: 600; cursor: pointer; transition: all 100ms;
  }
  .btn-danger-ghost:hover { background: rgba(239,68,68,.1); border-color: rgba(239,68,68,.5); }
  .btn-danger-confirm {
    padding: 6px 12px; border-radius: var(--r); border: 1px solid rgba(239,68,68,.5);
    background: rgba(239,68,68,.12); color: var(--p0); font-size: 12px; font-weight: 700; cursor: pointer;
  }
</style>
