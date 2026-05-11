<script lang="ts">
  import { onMount } from "svelte";
  import { items, dashboardStats, activeView, addToast, issueFilter, attentionRules, currentUser } from "../stores/app";
  import { getDashboardStats, listItems, updateItem, listBlockedTasks, updateTask } from "../api";
  import type { Item, Task } from "../types";

  let stats = { p0_count: 0, p1_count: 0, p2_count: 0, p3_count: 0, waiting_count: 0, waiting_overdue: 0, followups_today: 0, projects_at_risk: 0, projects_on_track: 0, total_projects: 0 };
  let blockedTasks: Task[] = [];

  onMount(async () => {
    try {
      stats = await getDashboardStats();
      dashboardStats.set(stats);
    } catch {}
    try {
      const it = await listItems();
      items.set(it);
    } catch {}
    try {
      blockedTasks = await listBlockedTasks();
    } catch {}
  });

  async function unblockTask(task: Task) {
    await updateTask({ id: task.id, status: "in_progress" });
    blockedTasks = blockedTasks.filter((t) => t.id !== task.id);
    addToast(`Unblocked: "${task.title.slice(0, 40)}"`, "✓");
  }

  $: needsAttention = (() => {
    const now = Date.now();
    const rules = $attentionRules;
    const staleDays = (rules.stale_p1_days || 7) * 24 * 60 * 60 * 1000;
    return $items.filter((i) => {
      if (i.status === "done") return false;
      if (i.type === "followup" && rules.overdue_followups) return i.due_at && new Date(i.due_at).getTime() < now;
      const ageDays = now - new Date(i.created_at).getTime();
      if (rules.p0 && i.priority === "p0") return true;
      if (rules.blocked && i.status === "blocked") return true;
      if (i.priority === "p1" && ageDays > staleDays) return true;
      return false;
    }).slice(0, 10);
  })();

  function attentionReason(item: any): { label: string; cls: string } {
    if (item.type === "followup") return { label: "Overdue follow-up", cls: "reason-overdue" };
    if (item.priority === "p0") return { label: "P0 Critical", cls: "reason-p0" };
    if (item.status === "blocked") return { label: "Blocked", cls: "reason-blocked" };
    return { label: "Stale P1", cls: "reason-stale" };
  }

  function daysSince(dateStr: string): string {
    const d = new Date(dateStr);
    const now = new Date();
    const diff = Math.floor((now.getTime() - d.getTime()) / (1000 * 60 * 60 * 24));
    return diff <= 0 ? "today" : `${diff}d`;
  }

  async function markDone(item: Item) {
    await updateItem({ id: item.id, status: "done" });
    items.update((list) => list.map((i) => (i.id === item.id ? { ...i, status: "done" as const } : i)));
    stats = await getDashboardStats();
    addToast(`Marked done: "${item.title.slice(0, 40)}"`, "✓");
  }

  const now = new Date();
  const greeting = now.getHours() < 12 ? "Good morning" : now.getHours() < 17 ? "Good afternoon" : "Good evening";
  const dateStr = now.toLocaleDateString("en-US", { weekday: "long", day: "numeric", month: "long", year: "numeric" });
</script>

<div class="page-header">
  <div class="page-title">{greeting}{$currentUser?.name ? `, ${$currentUser.name}` : ''}.</div>
  <div class="page-sub">{dateStr}{stats.p0_count > 0 ? ` · ${stats.p0_count} P0 issues need immediate attention` : ""}</div>
</div>

<div class="summary-row">
  <button class="stat-card red" on:click={() => { issueFilter.set("all"); activeView.set("issues"); }}>
    <div class="sc-label">P0 Critical</div>
    <div class="sc-num">{stats.p0_count}</div>
    <div class="sc-sub">Release blockers</div>
  </button>
  <button class="stat-card orange" on:click={() => { issueFilter.set("all"); activeView.set("issues"); }}>
    <div class="sc-label">P1 High</div>
    <div class="sc-num">{stats.p1_count}</div>
    <div class="sc-sub">This sprint</div>
  </button>
  <button class="stat-card yellow" on:click={() => activeView.set("followups")}>
    <div class="sc-label">Follow-ups</div>
    <div class="sc-num">{$items.filter(i => i.type === 'followup' && i.status !== 'done').length}</div>
    <div class="sc-sub">pending dependencies</div>
  </button>
  <button class="stat-card gray" on:click={() => { issueFilter.set("all"); activeView.set("issues"); }}>
    <div class="sc-label">Blocked</div>
    <div class="sc-num">{$items.filter(i => i.status === 'blocked' && i.status !== 'done').length}</div>
    <div class="sc-sub">need action</div>
  </button>
  <button class="stat-card green" on:click={() => activeView.set('projects')}>
    <div class="sc-label">Active Projects</div>
    <div class="sc-num">{stats.total_projects}</div>
    <div class="sc-sub">{stats.total_projects === 0 ? 'none yet' : 'in progress'}</div>
  </button>
  <button class="stat-card indigo" on:click={() => activeView.set('checkin')}>
    <div class="sc-label">Pending on Me</div>
    <div class="sc-num">{$items.filter(i => i.status === 'pending_me').length}</div>
    <div class="sc-sub">my dependencies</div>
  </button>
</div>

<div class="section-hdr">
  <div class="section-ttl">Needs Attention</div>
  <button class="section-link" on:click={() => { issueFilter.set("all"); activeView.set("issues"); }}>All issues →</button>
</div>
<div class="attn-cards">
  {#each needsAttention as item}
    {@const reason = attentionReason(item)}
    <div class="attn-card">
      <div class="attn-card-left">
        <div class="attn-proj">{item.project_name || (item.type === 'followup' ? 'Follow-up' : 'No project')}</div>
        <div class="attn-title">{item.title}</div>
        <div class="attn-badges">
          <span class="reason-badge {reason.cls}">{reason.label}</span>
          <span class="sev-pill {item.priority}">{item.priority.toUpperCase()}</span>
          <span class="aging-txt" class:hot={parseInt(daysSince(item.created_at)) > 5}>{daysSince(item.created_at)}</span>
        </div>
      </div>
      <div class="qa-actions">
        <button class="qa-btn done" on:click={() => markDone(item)}>✓</button>
      </div>
    </div>
  {:else}
    <div class="empty-state">No items need attention right now</div>
  {/each}
</div>

{#if blockedTasks.length > 0}
<div class="section-hdr">
  <div class="section-ttl">Project Blockers</div>
  <button class="section-link" on:click={() => activeView.set("projects")}>All projects →</button>
</div>
<div class="blocker-cards">
  {#each blockedTasks as task}
    <div class="blocker-card">
      <div class="blocker-left">
        <div class="blocker-proj">{task.project_name || "Unknown project"}</div>
        <div class="blocker-title">{task.title}</div>
      </div>
      <div class="qa-actions">
        <button class="qa-btn unblock" on:click={() => unblockTask(task)} title="Mark unblocked">→</button>
      </div>
    </div>
  {/each}
</div>
{/if}

<style>
  .page-header { margin-bottom: 18px; }
  .page-title { font-size: 17px; font-weight: 750; letter-spacing: -0.03em; }
  .page-sub { font-size: 12px; color: var(--fg-3); margin-top: 3px; }
  .section-hdr { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; margin-top: 22px; }
  .section-ttl { font-size: 12px; font-weight: 650; letter-spacing: -0.01em; }
  .section-link { font-size: 11px; color: var(--orange); cursor: pointer; font-weight: 500; background: none; border: none; }
  .section-link:hover { text-decoration: underline; }

  .summary-row { display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 8px; margin-bottom: 4px; }
  .stat-card {
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg);
    padding: 12px 14px; cursor: pointer; transition: all 150ms;
    display: flex; flex-direction: column; gap: 3px; position: relative; overflow: hidden;
    text-align: left;
  }
  .stat-card::after { content: ''; position: absolute; inset-block-start: 0; inset-inline-start: 0; inset-inline-end: 0; height: 2px; }
  .stat-card.orange::after { background: var(--orange); }
  .stat-card.red::after { background: var(--p0); }
  .stat-card.yellow::after { background: var(--p2); }
  .stat-card.green::after { background: var(--green); }
  .stat-card.gray::after { background: var(--fg-4); }
  .stat-card.indigo::after { background: #6366f1; }
  .stat-card:hover { border-color: var(--border-2); transform: translateY(-1px); box-shadow: var(--shadow); }
  .sc-label { font-size: 10px; color: var(--fg-3); font-weight: 500; }
  .sc-num { font-size: 26px; font-weight: 800; letter-spacing: -0.05em; font-variant-numeric: tabular-nums; line-height: 1; }
  .sc-sub { font-size: 10px; color: var(--fg-4); }

  .attn-cards { display: flex; flex-direction: column; gap: 6px; }
  .attn-card {
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg);
    padding: 10px 12px; display: flex; align-items: flex-start; gap: 10px;
    transition: all 120ms; cursor: pointer;
  }
  .attn-card:hover { border-color: var(--border-2); box-shadow: var(--shadow); }
  .attn-card-left { flex: 1; min-width: 0; }
  .attn-proj { font-size: 9px; color: var(--fg-4); text-transform: uppercase; letter-spacing: 0.06em; font-weight: 600; margin-bottom: 1px; }
  .attn-title { font-size: 12px; font-weight: 550; color: var(--fg); margin-bottom: 4px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .attn-badges { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
  .aging-txt { font-size: 10px; color: var(--fg-4); }
  .aging-txt.hot { color: var(--p0); font-weight: 600; }
  .reason-badge {
    font-size: 9px; font-weight: 700; padding: 2px 7px; border-radius: 99px;
    text-transform: uppercase; letter-spacing: 0.04em;
  }
  .reason-p0 { background: var(--p0-bg); color: var(--p0); }
  .reason-blocked { background: rgba(239,68,68,.12); color: #f87171; }
  .reason-overdue { background: rgba(234,179,8,.12); color: var(--p2); }
  .reason-stale { background: var(--surface-2); color: var(--fg-3); }
  .qa-actions { display: flex; gap: 4px; flex-shrink: 0; }
  .qa-btn {
    width: 26px; height: 26px; border-radius: 6px; border: 1px solid var(--border);
    color: var(--fg-3); display: flex; align-items: center; justify-content: center;
    transition: all 100ms; font-size: 11px; cursor: pointer; background: none;
  }
  .qa-btn:hover { background: var(--surface-2); color: var(--fg); border-color: var(--border-2); }
  .qa-btn.done:hover { background: var(--green-bg); border-color: var(--green); color: var(--green); }
  .qa-btn.unblock:hover { background: var(--orange-bg); border-color: var(--orange); color: var(--orange); }
  .empty-state { font-size: 12px; color: var(--fg-4); padding: 16px 0; text-align: center; }

  .blocker-cards { display: flex; flex-direction: column; gap: 6px; }
  .blocker-card {
    background: rgba(239,68,68,.04); border: 1px solid rgba(239,68,68,.2); border-radius: var(--r-lg);
    padding: 10px 12px; display: flex; align-items: center; gap: 10px; transition: all 120ms;
  }
  .blocker-card:hover { border-color: rgba(239,68,68,.4); }
  .blocker-left { flex: 1; min-width: 0; }
  .blocker-proj { font-size: 9px; color: #f87171; text-transform: uppercase; letter-spacing: 0.06em; font-weight: 700; margin-bottom: 1px; }
  .blocker-title { font-size: 12px; font-weight: 550; color: var(--fg); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
