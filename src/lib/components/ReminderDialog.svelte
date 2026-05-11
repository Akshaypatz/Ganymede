<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { TUNES, playTune } from "../tunes";
  import type { TuneId } from "../tunes";
  import { createReminder } from "../api";
  import { addToast } from "../stores/app";

  export let itemId: string;
  export let itemTitle: string;

  const dispatch = createEventDispatcher<{ close: void; created: void }>();

  // Default to "1 hour from now" rounded to the nearest 5 min
  function defaultDt() {
    const d = new Date(Date.now() + 60 * 60 * 1000);
    d.setSeconds(0, 0);
    d.setMinutes(Math.ceil(d.getMinutes() / 5) * 5);
    return d.toISOString().slice(0, 16); // "YYYY-MM-DDTHH:MM"
  }

  let dueAt  = defaultDt();
  let tune: TuneId = "bell";
  let label  = "";
  let saving = false;
  let err    = "";

  const QUICK = [
    { label: "In 30 min",        mins: 30   },
    { label: "In 1 hour",        mins: 60   },
    { label: "In 2 hours",       mins: 120  },
    { label: "Tomorrow 9 am",    special: "tomorrow9" },
    { label: "Next Monday 9 am", special: "nextmon"  },
  ];

  function applyQuick(q: typeof QUICK[number]) {
    const d = new Date();
    if ("mins" in q) {
      d.setTime(Date.now() + q.mins * 60_000);
    } else if (q.special === "tomorrow9") {
      d.setDate(d.getDate() + 1);
      d.setHours(9, 0, 0, 0);
    } else if (q.special === "nextmon") {
      const day = d.getDay(); // 0=Sun
      d.setDate(d.getDate() + ((8 - day) % 7 || 7));
      d.setHours(9, 0, 0, 0);
    }
    dueAt = d.toISOString().slice(0, 16);
  }

  async function save() {
    err = "";
    if (!dueAt) { err = "Please pick a date and time."; return; }
    saving = true;
    try {
      await createReminder({ item_id: itemId, due_at: new Date(dueAt).toISOString(), tune, label: label.trim() });
      addToast(`Reminder set for "${itemTitle.slice(0, 35)}"`, "✓");
      dispatch("created");
      dispatch("close");
    } catch (e: any) {
      err = String(e);
    } finally {
      saving = false;
    }
  }

  function close() { dispatch("close"); }
</script>

<svelte:window on:keydown={(e) => { if (e.key === 'Escape') { e.preventDefault(); close(); } }} />

<div class="overlay" on:click|self={close} role="dialog">
  <div class="modal reminder-modal">
    <div class="modal-header">
      <span class="modal-title">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" style="vertical-align:-2px;margin-right:6px;"><path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 01-3.46 0"/></svg>Set Reminder
      </span>
      <button class="btn-icon" on:click={close}>✕</button>
    </div>

    <div class="modal-body">
      <div class="item-context">For: <strong>{itemTitle}</strong></div>
      {#if err}<div class="form-error">{err}</div>{/if}

      <!-- Quick picks -->
      <div class="field-col">
        <div class="field-label">Quick pick</div>
        <div class="quick-row">
          {#each QUICK as q}
            <button class="quick-btn" on:click={() => applyQuick(q)}>{q.label}</button>
          {/each}
        </div>
      </div>

      <!-- Date/time -->
      <div class="field-col">
        <div class="field-label">Date &amp; time <span class="req">*</span></div>
        <input type="datetime-local" bind:value={dueAt} class="field-input dt-input" />
      </div>

      <!-- Tune -->
      <div class="field-col">
        <div class="field-label">Notification tune</div>
        <div class="tune-row">
          {#each TUNES as t}
            <button
              class="tune-btn"
              class:active={tune === t.id}
              on:click={() => tune = t.id as TuneId}
            >
              <span class="tune-emoji">{t.emoji}</span>
              <span class="tune-name">{t.name}</span>
            </button>
          {/each}
        </div>
        {#if tune !== "none"}
          <button class="preview-btn" on:click={() => playTune(tune)}>▶ Preview "{TUNES.find(t => t.id === tune)?.name}"</button>
        {/if}
      </div>

      <!-- Label -->
      <div class="field-col">
        <div class="field-label">Note (optional)</div>
        <input bind:value={label} class="field-input" placeholder="e.g. Follow up with Rahul" maxlength="80" />
      </div>
    </div>

    <div class="modal-footer">
      <button class="btn btn-ghost" on:click={close}>Cancel</button>
      <button class="btn btn-primary" on:click={save} disabled={saving}>
        {saving ? "Saving…" : "Set Reminder"}
      </button>
    </div>
  </div>
</div>

<style>
  .reminder-modal { width: 460px; max-height: 88vh; overflow-y: auto; }
  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 18px; border-bottom: 1px solid var(--border);
  }
  .modal-title { font-size: 13px; font-weight: 700; }
  .modal-body { padding: 18px; display: flex; flex-direction: column; gap: 16px; }
  .modal-footer { padding: 12px 18px; border-top: 1px solid var(--border); display: flex; gap: 8px; justify-content: flex-end; }

  .item-context { font-size: 11px; color: var(--fg-3); }
  .item-context strong { color: var(--fg); }
  .form-error { font-size: 11px; color: var(--p0); background: rgba(239,68,68,.08); padding: 6px 10px; border-radius: var(--r); }
  .field-col { display: flex; flex-direction: column; gap: 6px; }
  .field-label { font-size: 10px; font-weight: 700; color: var(--fg-4); text-transform: uppercase; letter-spacing: 0.05em; }
  .req { color: var(--p0); }
  .field-input { padding: 7px 10px; background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg); outline: none; transition: border-color 120ms; }
  .field-input:focus { border-color: var(--orange); }
  .dt-input { font-family: inherit; }

  .quick-row { display: flex; flex-wrap: wrap; gap: 5px; }
  .quick-btn {
    padding: 4px 10px; font-size: 11px; border-radius: var(--r); cursor: pointer;
    background: var(--surface-2); border: 1px solid var(--border); color: var(--fg-3);
    transition: all 100ms;
  }
  .quick-btn:hover { border-color: var(--orange); color: var(--orange); }

  .tune-row { display: flex; flex-wrap: wrap; gap: 6px; }
  .tune-btn {
    display: flex; flex-direction: column; align-items: center; gap: 2px;
    padding: 6px 10px; min-width: 58px;
    border-radius: var(--r); border: 1px solid var(--border); cursor: pointer;
    background: var(--surface-2); transition: all 100ms;
  }
  .tune-btn:hover { border-color: var(--border-2); }
  .tune-btn.active { border-color: var(--orange); background: rgba(249,115,22,.08); }
  .tune-emoji { font-size: 16px; line-height: 1; }
  .tune-name { font-size: 9px; font-weight: 600; color: var(--fg-3); }
  .tune-btn.active .tune-name { color: var(--orange); }

  .preview-btn {
    align-self: flex-start; padding: 3px 10px; font-size: 10px; border-radius: var(--r);
    background: none; border: 1px solid var(--border); color: var(--fg-3); cursor: pointer;
    transition: all 100ms;
  }
  .preview-btn:hover { border-color: var(--orange); color: var(--orange); }

  /* inherit overlay/modal styles from global */
  .overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,.45); display: flex;
    align-items: center; justify-content: center; z-index: 200;
  }
  .modal {
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-xl);
    box-shadow: 0 24px 48px rgba(0,0,0,.22);
  }
  .btn-icon { background: none; border: none; color: var(--fg-4); cursor: pointer; font-size: 14px; }
  .btn-icon:hover { color: var(--fg); }
  .btn { padding: 6px 14px; border-radius: var(--r); font-size: 12px; font-weight: 600; cursor: pointer; border: 1px solid transparent; transition: all 100ms; }
  .btn-ghost { background: none; border-color: var(--border); color: var(--fg-3); }
  .btn-ghost:hover { color: var(--fg); border-color: var(--border-2); }
  .btn-primary { background: var(--orange); color: #fff; border-color: var(--orange); }
  .btn-primary:hover { opacity: .88; }
  .btn-primary:disabled { opacity: .5; cursor: not-allowed; }
</style>
