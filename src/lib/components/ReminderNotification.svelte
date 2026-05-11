<script lang="ts">
  import type { Reminder } from "../types";
  import { playTune } from "../tunes";
  import type { TuneId } from "../tunes";
  import { completeReminder, snoozeReminder } from "../api";
  import { onMount, createEventDispatcher } from "svelte";

  export let reminders: Reminder[];

  const dispatch = createEventDispatcher<{ dismiss: string }>();

  onMount(() => {
    // Play the tune for the first (highest-priority) reminder
    if (reminders.length > 0) {
      try { playTune(reminders[0].tune as TuneId); } catch {}
    }
  });

  async function dismiss(r: Reminder) {
    await completeReminder(r.id).catch(() => {});
    dispatch("dismiss", r.id);
  }

  async function snooze(r: Reminder, mins: number) {
    await snoozeReminder(r.id, mins).catch(() => {});
    dispatch("dismiss", r.id);
  }
</script>

<div class="notif-stack">
  {#each reminders as r (r.id)}
    <div class="notif-card" role="alert">
      <div class="notif-header">
        <span class="notif-icon"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 01-3.46 0"/></svg></span>
        <div class="notif-text">
          <div class="notif-title">{r.item_title || "Reminder"}</div>
          {#if r.label}<div class="notif-label">{r.label}</div>{/if}
          <div class="notif-time">Due: {new Date(r.due_at).toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit" })}</div>
        </div>
        <button class="notif-close" on:click={() => dismiss(r)} title="Dismiss">✕</button>
      </div>
      <div class="notif-actions">
        <button class="snooze-btn" on:click={() => snooze(r, 5)}>Snooze 5 min</button>
        <button class="snooze-btn" on:click={() => snooze(r, 60)}>1 hour</button>
        <button class="dismiss-btn" on:click={() => dismiss(r)}>Dismiss</button>
      </div>
    </div>
  {/each}
</div>

<style>
  .notif-stack {
    position: fixed; top: 20px; right: 20px; z-index: 500;
    display: flex; flex-direction: column; gap: 10px; max-width: 340px;
  }
  .notif-card {
    background: var(--surface); border: 1px solid var(--orange);
    border-radius: var(--r-xl); box-shadow: 0 8px 28px rgba(0,0,0,.22);
    overflow: hidden;
    animation: notif-in 220ms cubic-bezier(.16,1,.3,1);
  }
  @keyframes notif-in {
    from { opacity: 0; transform: translateY(-16px) scale(.95); }
    to   { opacity: 1; transform: none; }
  }
  .notif-header {
    display: flex; align-items: flex-start; gap: 10px;
    padding: 12px 12px 8px; border-bottom: 1px solid var(--border);
  }
  .notif-icon { font-size: 20px; flex-shrink: 0; margin-top: 1px; }
  .notif-text { flex: 1; min-width: 0; }
  .notif-title { font-size: 12px; font-weight: 700; color: var(--fg); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .notif-label { font-size: 11px; color: var(--fg-3); margin-top: 2px; }
  .notif-time { font-size: 10px; color: var(--orange); font-weight: 600; margin-top: 3px; }
  .notif-close { background: none; border: none; color: var(--fg-4); cursor: pointer; font-size: 12px; flex-shrink: 0; padding: 2px 4px; }
  .notif-close:hover { color: var(--fg); }
  .notif-actions { display: flex; gap: 6px; padding: 8px 12px; }
  .snooze-btn {
    flex: 1; padding: 4px 0; font-size: 10px; font-weight: 600; border-radius: var(--r);
    background: var(--surface-2); border: 1px solid var(--border); color: var(--fg-3); cursor: pointer;
    transition: all 100ms;
  }
  .snooze-btn:hover { border-color: var(--orange); color: var(--orange); }
  .dismiss-btn {
    padding: 4px 10px; font-size: 10px; font-weight: 700; border-radius: var(--r);
    background: rgba(249,115,22,.1); border: 1px solid rgba(249,115,22,.3); color: var(--orange);
    cursor: pointer; transition: all 100ms;
  }
  .dismiss-btn:hover { background: rgba(249,115,22,.2); }
</style>
