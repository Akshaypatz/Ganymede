<script lang="ts">
  import { onMount } from "svelte";
  import {
    checkinReport,
    activeView,
    aiMessages,
    aiConversationId,
    addToast,
    items,
  } from "../stores/app";
  import { getCheckinReport, snoozeCheckinItem, updateItem } from "../api";
  import type { CheckinItem, CheckinProject } from "../types";

  let loading = false;
  let lastRefresh = "";
  let snoozeOpenId: string | null = null;
  let deleteConfirmId: string | null = null;
  let showConfig = false;

  // Section collapse state
  let collapsed: Record<string, boolean> = {};

  // ── Brief configuration ─────────────────────────────
  interface BriefConfig {
    showPendingMe: boolean;
    showOverdue:   boolean;
    showBlocked:   boolean;
    showStale:     boolean;
    showDueSoon:   boolean;
    showAtRisk:    boolean;
    staleAfterDays:  number;
    dueSoonWithinDays: number;
  }
  const CONFIG_KEY = "ganymede_brief_config";
  const DEFAULT_CONFIG: BriefConfig = {
    showPendingMe: true,
    showOverdue:   true,
    showBlocked:   true,
    showStale:     true,
    showDueSoon:   true,
    showAtRisk:    true,
    staleAfterDays:    7,
    dueSoonWithinDays: 2,
  };
  let cfg: BriefConfig = { ...DEFAULT_CONFIG };

  function loadConfig() {
    try {
      const raw = localStorage.getItem(CONFIG_KEY);
      if (raw) cfg = { ...DEFAULT_CONFIG, ...JSON.parse(raw) };
    } catch {}
  }
  function saveConfig() {
    try { localStorage.setItem(CONFIG_KEY, JSON.stringify(cfg)); } catch {}
  }
  function resetConfig() {
    cfg = { ...DEFAULT_CONFIG };
    saveConfig();
  }

  onMount(() => {
    loadConfig();
    refresh();
  });

  async function refresh() {
    loading = true;
    try {
      const report = await getCheckinReport();
      checkinReport.set(report);
      lastRefresh = new Date().toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit" });
    } catch (e: any) {
      addToast(`Check-in error: ${e}`, "✗");
    }
    loading = false;
  }

  async function resolveItem(item: CheckinItem) {
    await updateItem({ id: item.id, status: "done" });
    items.update(list => list.map(i => i.id === item.id ? { ...i, status: "done" as any } : i));
    addToast(`Resolved: "${item.title.slice(0, 40)}"`, "✓");
    await refresh();
  }

  async function snooze(item: CheckinItem, days: number) {
    await snoozeCheckinItem(item.id, days);
    snoozeOpenId = null;
    addToast(`Snoozed for ${days} day${days > 1 ? "s" : ""}`, "\u23f8");
    await refresh();
  }

  function openInAi(item: CheckinItem) {
    const reason = item.reason === "overdue"
      ? "is overdue"
      : item.reason === "blocked"
      ? `has been blocked for ${item.age_days} days`
      : item.reason === "stale"
      ? `hasn't been updated in ${item.age_days} days`
      : "is due soon";
    const msg = `"${item.title}" (${item.item_type}, ${item.priority}) ${reason}. What should I do next?`;
    aiConversationId.set(null);
    aiMessages.set([]);
    activeView.set("ai");
    setTimeout(() => {
      window.dispatchEvent(new CustomEvent("ganymede:prefill-ai", { detail: msg }));
    }, 80);
  }

  function briefMe() {
    const r = $checkinReport;
    if (!r || r.total_attention === 0) return;
    const lines: string[] = [];
    if (r.overdue.length) lines.push(`${r.overdue.length} overdue (${r.overdue.slice(0,3).map(i => `"${i.title}"`).join(", ")})`);
    if (r.blocked.length) lines.push(`${r.blocked.length} blocked (${r.blocked.slice(0,3).map(i => `"${i.title}"`).join(", ")})`);
    if (r.stale.length) lines.push(`${r.stale.length} stale`);
    if (r.due_soon.length) lines.push(`${r.due_soon.length} due within 2 days`);
    if (r.pending_me.length) lines.push(`${r.pending_me.length} pending on me (${r.pending_me.slice(0,3).map(i => `"${i.title}"`).join(", ")})`);
    if (r.at_risk_projects.length) lines.push(`${r.at_risk_projects.length} projects to watch`);
    const msg = `I have ${r.total_attention} items needing attention: ${lines.join(", ")}. Give me a prioritized action plan and flag what I should personally unblock today.`;
    aiConversationId.set(null);
    aiMessages.set([]);
    activeView.set("ai");
    setTimeout(() => {
      window.dispatchEvent(new CustomEvent("ganymede:prefill-ai", { detail: msg }));
    }, 80);
  }

  function toggle(key: string) {
    collapsed = { ...collapsed, [key]: !collapsed[key] };
  }

  // Recompute total_attention to reflect only visible sections (respects cfg)
  $: if ($checkinReport && cfg) {
    const visible =
      (cfg.showOverdue   ? $checkinReport.overdue.length         : 0) +
      (cfg.showBlocked   ? $checkinReport.blocked.length         : 0) +
      (cfg.showStale     ? $checkinReport.stale.length           : 0) +
      (cfg.showDueSoon   ? $checkinReport.due_soon.length        : 0) +
      (cfg.showPendingMe ? $checkinReport.pending_me.length      : 0) +
      (cfg.showAtRisk    ? $checkinReport.at_risk_projects.length : 0);
    if ($checkinReport.total_attention !== visible) {
      checkinReport.update(r => r ? { ...r, total_attention: visible } : r);
    }
  }

  function priorityColor(p: string) {
    if (p === "p0") return "var(--p0)";
    if (p === "p1") return "var(--p1)";
    if (p === "p2") return "var(--p2)";
    return "var(--fg-4)";
  }

  function priorityBg(p: string) {
    if (p === "p0") return "var(--p0-bg)";
    if (p === "p1") return "var(--p1-bg)";
    if (p === "p2") return "var(--p2-bg)";
    return "var(--surface-2)";
  }

  function stageLabel(s: string) {
    return s.replace(/_/g, " ").replace(/\b\w/g, c => c.toUpperCase());
  }

  $: report = $checkinReport;
  $: today = new Date().toLocaleDateString("en-US", { weekday: "long", month: "long", day: "numeric" });

  const SECTIONS = [
    { key: "pending_me", icon: "user-clock",    label: "Pending on Me", emptyMsg: "Nothing pending on you" },
    { key: "overdue",    icon: "circle-alert",  label: "Overdue",       emptyMsg: "Nothing overdue" },
    { key: "blocked",    icon: "circle-slash",  label: "Blocked",       emptyMsg: "No blocked items" },
    { key: "stale",      icon: "clock",         label: "Stale",         emptyMsg: "All items are active" },
    { key: "due_soon",   icon: "calendar",      label: "Due Soon",      emptyMsg: "Nothing due soon" },
  ] as const;

  const SECTION_ICONS: Record<string, string> = {
    "user-clock":   `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/><circle cx="18.5" cy="17.5" r="3.5"/><polyline points="18.5 16 18.5 17.5 19.5 18.5"/></svg>`,
    "circle-alert": `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>`,
    "circle-slash": `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>`,
    "clock":        `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>`,
    "calendar":     `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>`,
  };
</script>

<div class="checkin-page">
  <!-- Header -->
  <div class="ci-header">
    <div class="ci-header-left">
      <div class="ci-logo-wrap" aria-hidden="true">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="var(--orange)" stroke-width="2">
          <rect x="3" y="4" width="18" height="18" rx="2"/>
          <line x1="16" y1="2" x2="16" y2="6"/>
          <line x1="8" y1="2" x2="8" y2="6"/>
          <line x1="3" y1="10" x2="21" y2="10"/>
          <path d="M8 14h.01M12 14h.01M16 14h.01M8 18h.01M12 18h.01"/>
        </svg>
      </div>
      <div>
        <div class="ci-title">Daily Brief</div>
        <div class="ci-date">{today}</div>
      </div>
    </div>
    <div class="ci-header-right">
      {#if report && report.total_attention > 0}
        <button class="btn-brief" on:click={briefMe}>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg>
          Ask AI to brief me
        </button>
      {/if}
      <button class="btn-configure" class:active={showConfig} on:click={() => showConfig = !showConfig} title="Configure Daily Brief">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
        Configure
      </button>
      <button class="btn-refresh" on:click={refresh} class:spinning={loading} title="Refresh">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="1 4 1 10 7 10"/><path d="M3.51 15a9 9 0 102.13-9.36L1 10"/></svg>
      </button>
    </div>
  </div>

  <!-- Configuration panel -->
  {#if showConfig}
  <div class="ci-config-panel">
    <div class="ci-config-title">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
      What to show in Daily Brief
    </div>
    <div class="ci-config-grid">
      <label class="ci-toggle">
        <input type="checkbox" bind:checked={cfg.showPendingMe} on:change={saveConfig} />
        <span class="ci-toggle-track"><span class="ci-toggle-thumb"></span></span>
        <span class="ci-toggle-label">
          <span class="ci-toggle-name">Pending on Me</span>
          <span class="ci-toggle-desc">Items you've pinned as waiting on your action</span>
        </span>
      </label>
      <label class="ci-toggle">
        <input type="checkbox" bind:checked={cfg.showOverdue} on:change={saveConfig} />
        <span class="ci-toggle-track"><span class="ci-toggle-thumb"></span></span>
        <span class="ci-toggle-label">
          <span class="ci-toggle-name">Overdue</span>
          <span class="ci-toggle-desc">Items past their due date</span>
        </span>
      </label>
      <label class="ci-toggle">
        <input type="checkbox" bind:checked={cfg.showBlocked} on:change={saveConfig} />
        <span class="ci-toggle-track"><span class="ci-toggle-thumb"></span></span>
        <span class="ci-toggle-label">
          <span class="ci-toggle-name">Blocked</span>
          <span class="ci-toggle-desc">Items stuck in blocked status</span>
        </span>
      </label>
      <label class="ci-toggle">
        <input type="checkbox" bind:checked={cfg.showStale} on:change={saveConfig} />
        <span class="ci-toggle-track"><span class="ci-toggle-thumb"></span></span>
        <span class="ci-toggle-label">
          <span class="ci-toggle-name">Stale items</span>
          <span class="ci-toggle-desc">No updates for more than
            <input class="ci-inline-num" type="number" min="3" max="30" bind:value={cfg.staleAfterDays} on:change={saveConfig} />
            days</span>
        </span>
      </label>
      <label class="ci-toggle">
        <input type="checkbox" bind:checked={cfg.showDueSoon} on:change={saveConfig} />
        <span class="ci-toggle-track"><span class="ci-toggle-thumb"></span></span>
        <span class="ci-toggle-label">
          <span class="ci-toggle-name">Due soon</span>
          <span class="ci-toggle-desc">Due within the next
            <input class="ci-inline-num" type="number" min="1" max="14" bind:value={cfg.dueSoonWithinDays} on:change={saveConfig} />
            days</span>
        </span>
      </label>
      <label class="ci-toggle">
        <input type="checkbox" bind:checked={cfg.showAtRisk} on:change={saveConfig} />
        <span class="ci-toggle-track"><span class="ci-toggle-thumb"></span></span>
        <span class="ci-toggle-label">
          <span class="ci-toggle-name">Projects to Watch</span>
          <span class="ci-toggle-desc">Projects with amber/red health status</span>
        </span>
      </label>
    </div>
    <div class="ci-config-footer">
      <button class="ci-reset-btn" on:click={resetConfig}>Reset to defaults</button>
      <button class="ci-config-close" on:click={() => showConfig = false}>Done</button>
    </div>
  </div>
  {/if}

  <!-- Summary bar -->
  {#if report}
    {#if report.total_attention === 0}
      <div class="ci-all-clear">
        <span class="ci-all-clear-icon">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10" stroke="var(--green)"/><polyline points="9 12 11 14 15 10" stroke="var(--green)" stroke-width="2.5"/></svg>
        </span>
        <div>
          <div class="ci-all-clear-title">All clear!</div>
          <div class="ci-all-clear-sub">No items need your attention right now. Check back later.</div>
        </div>
      </div>
    {:else}
      <div class="ci-summary-bar">
        {#if cfg.showOverdue && report.overdue.length}
          <div class="ci-stat ci-stat-red">
            <span class="ci-stat-num">{report.overdue.length}</span>
            <span class="ci-stat-label">Overdue</span>
          </div>
        {/if}
        {#if cfg.showBlocked && report.blocked.length}
          <div class="ci-stat ci-stat-orange">
            <span class="ci-stat-num">{report.blocked.length}</span>
            <span class="ci-stat-label">Blocked</span>
          </div>
        {/if}
        {#if cfg.showStale && report.stale.length}
          <div class="ci-stat ci-stat-yellow">
            <span class="ci-stat-num">{report.stale.length}</span>
            <span class="ci-stat-label">Stale</span>
          </div>
        {/if}
        {#if cfg.showDueSoon && report.due_soon.length}
          <div class="ci-stat ci-stat-blue">
            <span class="ci-stat-num">{report.due_soon.length}</span>
            <span class="ci-stat-label">Due Soon</span>
          </div>
        {/if}
        {#if cfg.showAtRisk && report.at_risk_projects.length}
          <div class="ci-stat ci-stat-purple">
            <span class="ci-stat-num">{report.at_risk_projects.length}</span>
            <span class="ci-stat-label">Projects to Watch</span>
          </div>
        {/if}
        {#if cfg.showPendingMe && report.pending_me.length}
          <div class="ci-stat ci-stat-indigo">
            <span class="ci-stat-num">{report.pending_me.length}</span>
            <span class="ci-stat-label">Pending on Me</span>
          </div>
        {/if}
      </div>
    {/if}
  {:else}
    <div class="ci-loading">Loading check-in report…</div>
  {/if}

  <!-- Item sections -->
  {#if report}
    {#each SECTIONS as section}
      {@const sectionItems = report[section.key] as CheckinItem[]}
      {@const sectionEnabled = section.key === 'pending_me' ? cfg.showPendingMe
        : section.key === 'overdue'  ? cfg.showOverdue
        : section.key === 'blocked'  ? cfg.showBlocked
        : section.key === 'stale'    ? cfg.showStale
        : section.key === 'due_soon' ? cfg.showDueSoon
        : true}
      {#if sectionEnabled && sectionItems.length > 0}
        <div class="ci-section">
          <button class="ci-section-hdr" on:click={() => toggle(section.key)}>
            <span class="ci-section-icon ci-section-icon-svg">{@html SECTION_ICONS[section.icon]}</span>
            <span class="ci-section-label">{section.label}</span>
            <span class="ci-section-count">{sectionItems.length}</span>
            <svg
              class="ci-chevron" class:open={!collapsed[section.key]}
              width="12" height="12" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2.5"
            ><polyline points="6 9 12 15 18 9"/></svg>
          </button>

          {#if !collapsed[section.key]}
            <div class="ci-item-list">
              {#each sectionItems as ci (ci.id)}
                <div class="ci-item" class:ci-item-overdue={ci.reason === 'overdue'} class:ci-item-blocked={ci.reason === 'blocked'}>
                  <!-- Left accent bar by reason -->
                  <div class="ci-item-accent" style="background:{
                    ci.reason === 'overdue' ? 'var(--p0)' :
                    ci.reason === 'blocked' ? 'var(--orange)' :
                    ci.reason === 'stale' ? '#eab308' :
                    ci.reason === 'pending_me' ? '#8b5cf6' : '#3b82f6'
                  }"></div>

                  <div class="ci-item-main">
                    <div class="ci-item-top">
                      <span class="ci-priority" style="background:{priorityBg(ci.priority)};color:{priorityColor(ci.priority)}">{ci.priority.toUpperCase()}</span>
                      <span class="ci-type-badge">{ci.item_type}</span>
                      <span class="ci-title">{ci.title}</span>
                    </div>
                    <div class="ci-item-meta">
                      {#if ci.project_name}
                        <span class="ci-meta-pill">
                          <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 3h6a4 4 0 014 4v14a3 3 0 00-3-3H2z"/><path d="M22 3h-6a4 4 0 00-4 4v14a3 3 0 013-3h7z"/></svg>
                          {ci.project_name}
                        </span>
                      {/if}
                      {#if ci.assignee}
                        <span class="ci-meta-pill">
                          <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                          {ci.assignee}
                        </span>
                      {/if}
                      {#if ci.reason === "overdue" && ci.due_at}
                        <span class="ci-meta-pill ci-pill-danger">Due {new Date(ci.due_at).toLocaleDateString("en-US", { month: "short", day: "numeric" })}</span>
                      {:else if ci.reason === "due_soon" && ci.due_at}
                        <span class="ci-meta-pill ci-pill-warn">Due {new Date(ci.due_at).toLocaleDateString("en-US", { month: "short", day: "numeric" })}</span>
                      {:else}
                        <span class="ci-meta-pill ci-pill-age">Updated {ci.age_days === 0 ? "today" : ci.age_days === 1 ? "yesterday" : `${ci.age_days}d ago`}</span>
                      {/if}
                    </div>
                  </div>

                  <!-- Actions -->
                  <div class="ci-item-actions">
                    <button class="ci-action-btn ci-action-ai" on:click={() => openInAi(ci)} title="Update via AI">
                      <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg>
                      Ask AI
                    </button>
                    <button class="ci-action-btn ci-action-done" on:click={() => resolveItem(ci)} title="Mark done">
                      <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
                      Resolve
                    </button>
                    <div class="ci-snooze-wrap">
                      <button class="ci-action-btn ci-action-snooze" on:click={() => snoozeOpenId = snoozeOpenId === ci.id ? null : ci.id} title="Snooze">
                        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                      </button>
                      {#if snoozeOpenId === ci.id}
                        <div class="ci-snooze-menu">
                          <button on:click={() => snooze(ci, 1)}>1 day</button>
                          <button on:click={() => snooze(ci, 2)}>2 days</button>
                          <button on:click={() => snooze(ci, 7)}>1 week</button>
                        </div>
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    {/each}

    <!-- Projects to Watch -->
    {#if cfg.showAtRisk && report.at_risk_projects.length > 0}
      <div class="ci-section">
        <button class="ci-section-hdr" on:click={() => toggle("projects")}>
          <span class="ci-section-icon ci-section-icon-svg"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg></span>
          <span class="ci-section-label">Projects to Watch</span>
          <span class="ci-section-count">{report.at_risk_projects.length}</span>
          <svg
            class="ci-chevron" class:open={!collapsed["projects"]}
            width="12" height="12" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2.5"
          ><polyline points="6 9 12 15 18 9"/></svg>
        </button>

        {#if !collapsed["projects"]}
          <div class="ci-item-list">
            {#each report.at_risk_projects as proj (proj.id)}
              <div class="ci-item">
                <div class="ci-item-accent" style="background:{proj.health === 'red' ? 'var(--p0)' : '#8b5cf6'}"></div>
                <div class="ci-item-main">
                  <div class="ci-item-top">
                    <span class="ci-health-dot" style="background:{proj.health === 'red' ? 'var(--p0)' : proj.health === 'amber' ? 'var(--orange)' : 'var(--green)'}"></span>
                    <span class="ci-title">{proj.name}</span>
                  </div>
                  <div class="ci-item-meta">
                    <span class="ci-meta-pill">{stageLabel(proj.stage)}</span>
                    <span class="ci-meta-pill ci-pill-age">
                      {proj.last_activity_days === 0 ? "Active today" : `No activity for ${proj.last_activity_days}d`}
                    </span>
                  </div>
                </div>
                <div class="ci-item-actions">
                  <button class="ci-action-btn ci-action-ai" on:click={() => {
                    const msg = `Project "${proj.name}" needs attention (health: ${proj.health}, ${proj.last_activity_days} days inactive). What should I check?`;
                    aiConversationId.set(null); aiMessages.set([]); activeView.set("ai");
                    setTimeout(() => window.dispatchEvent(new CustomEvent("ganymede:prefill-ai", { detail: msg })), 80);
                  }}>
                    <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg>
                    Ask AI
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  {/if}

  <!-- Footer -->
  {#if lastRefresh}
    <div class="ci-footer">Last refreshed at {lastRefresh}</div>
  {/if}
</div>

<style>
  .checkin-page { display: flex; flex-direction: column; gap: 16px; max-width: 860px; }

  /* Header */
  .ci-header {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 4px;
  }
  .ci-header-left { display: flex; align-items: center; gap: 14px; }
  .ci-logo-wrap {
    width: 44px; height: 44px; border-radius: 12px;
    background: var(--orange-bg);
    display: flex; align-items: center; justify-content: center;
    border: 1px solid rgba(249,115,22,.2);
    flex-shrink: 0;
  }
  .ci-title { font-size: 20px; font-weight: 780; letter-spacing: -0.03em; }
  .ci-date { font-size: 11px; color: var(--fg-4); margin-top: 2px; }
  .ci-header-right { display: flex; align-items: center; gap: 8px; }
  .btn-brief {
    display: flex; align-items: center; gap: 6px;
    padding: 7px 14px; border-radius: var(--r); border: none;
    background: linear-gradient(135deg, #fb923c, #f97316); color: #fff;
    font-size: 12px; font-weight: 600; cursor: pointer; transition: opacity 120ms;
  }
  .btn-brief:hover { opacity: 0.88; }
  .btn-refresh {
    width: 32px; height: 32px; display: flex; align-items: center; justify-content: center;
    border-radius: var(--r); border: 1px solid var(--border);
    background: var(--surface-2); color: var(--fg-3); cursor: pointer; transition: all 120ms;
  }
  .btn-refresh:hover { border-color: var(--orange); color: var(--orange); }
  .btn-configure {
    display: flex; align-items: center; gap: 6px;
    padding: 6px 12px; border-radius: var(--r); border: 1px solid var(--border);
    background: var(--surface-2); color: var(--fg-3); font-size: 11px; font-weight: 600;
    cursor: pointer; transition: all 120ms;
  }
  .btn-configure:hover, .btn-configure.active { border-color: var(--orange); color: var(--orange); background: var(--orange-bg); }
  .spinning { animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* Config panel */
  .ci-config-panel {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    padding: 18px 20px;
    animation: fadeSlideDown 180ms ease;
  }
  @keyframes fadeSlideDown {
    from { opacity: 0; transform: translateY(-6px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  .ci-config-title {
    display: flex; align-items: center; gap: 8px;
    font-size: 11px; font-weight: 700; color: var(--fg-3);
    text-transform: uppercase; letter-spacing: 0.06em;
    margin-bottom: 16px;
  }
  .ci-config-grid { display: flex; flex-direction: column; gap: 12px; }
  .ci-toggle {
    display: flex; align-items: flex-start; gap: 12px; cursor: pointer;
  }
  .ci-toggle input { display: none; }
  .ci-toggle-track {
    width: 34px; height: 18px; border-radius: 9px;
    background: var(--surface-2); border: 1.5px solid var(--border);
    position: relative; flex-shrink: 0; margin-top: 1px;
    transition: all 140ms;
  }
  .ci-toggle input:checked + .ci-toggle-track {
    background: var(--orange); border-color: var(--orange);
  }
  .ci-toggle-thumb {
    position: absolute; top: 1px; left: 1px;
    width: 12px; height: 12px; border-radius: 50%; background: var(--fg-4);
    transition: transform 140ms, background 140ms;
  }
  .ci-toggle input:checked + .ci-toggle-track .ci-toggle-thumb {
    transform: translateX(16px); background: #fff;
  }
  .ci-toggle-label { display: flex; flex-direction: column; gap: 1px; }
  .ci-toggle-name { font-size: 12px; font-weight: 600; color: var(--fg); }
  .ci-toggle-desc { font-size: 11px; color: var(--fg-4); line-height: 1.4; }
  .ci-inline-num {
    width: 38px; background: var(--surface-2); border: 1px solid var(--border);
    border-radius: 4px; padding: 1px 5px; font-size: 11px; color: var(--fg);
    text-align: center; appearance: textfield;
    display: inline-block; margin: 0 3px;
  }
  .ci-inline-num:focus { outline: none; border-color: var(--orange); }
  .ci-config-footer {
    display: flex; align-items: center; justify-content: space-between;
    margin-top: 16px; padding-top: 14px; border-top: 1px solid var(--border);
  }
  .ci-reset-btn {
    font-size: 11px; color: var(--fg-4); background: none; border: none;
    cursor: pointer; transition: color 120ms;
  }
  .ci-reset-btn:hover { color: var(--fg); }
  .ci-config-close {
    padding: 6px 16px; background: var(--orange); color: #fff;
    border: none; border-radius: var(--r); font-size: 11px; font-weight: 600;
    cursor: pointer; transition: opacity 120ms;
  }
  .ci-config-close:hover { opacity: 0.88; }

  /* Summary */
  .ci-summary-bar {
    display: flex; gap: 10px; flex-wrap: wrap;
  }
  .ci-stat {
    display: flex; align-items: center; gap: 6px;
    padding: 8px 14px; border-radius: var(--r-lg); border: 1px solid;
    font-size: 12px; font-weight: 600;
  }
  .ci-stat-num { font-size: 22px; font-weight: 800; line-height: 1; }
  .ci-stat-label { font-size: 11px; font-weight: 500; opacity: 0.85; }
  .ci-stat-red    { background: rgba(239,68,68,.08); border-color: rgba(239,68,68,.25); color: #ef4444; }
  .ci-stat-orange { background: rgba(249,115,22,.08); border-color: rgba(249,115,22,.25); color: var(--orange); }
  .ci-stat-yellow { background: rgba(234,179,8,.08); border-color: rgba(234,179,8,.25); color: #ca8a04; }
  .ci-stat-blue   { background: rgba(59,130,246,.08); border-color: rgba(59,130,246,.25); color: #3b82f6; }
  .ci-stat-purple { background: rgba(139,92,246,.08); border-color: rgba(139,92,246,.25); color: #8b5cf6; }
  .ci-stat-indigo { background: rgba(99,102,241,.08); border-color: rgba(99,102,241,.25); color: #6366f1; }

  .ci-all-clear {
    display: flex; align-items: center; gap: 14px;
    padding: 20px 22px; border-radius: var(--r-lg);
    background: rgba(34,197,94,.06); border: 1px solid rgba(34,197,94,.2);
  }
  .ci-all-clear-icon { font-size: 28px; }
  .ci-all-clear-title { font-size: 14px; font-weight: 700; color: var(--green); }
  .ci-all-clear-sub { font-size: 12px; color: var(--fg-3); margin-top: 2px; }
  .ci-loading { font-size: 12px; color: var(--fg-4); padding: 20px 0; }

  /* Section */
  .ci-section {
    background: var(--surface); border: 1px solid var(--border);
    border-radius: var(--r-lg); overflow: hidden;
  }
  .ci-section-hdr {
    width: 100%; display: flex; align-items: center; gap: 8px;
    padding: 11px 16px; background: none; border: none; cursor: pointer;
    text-align: left; transition: background 120ms;
  }
  .ci-section-hdr:hover { background: var(--surface-2); }
  .ci-section-icon { font-size: 14px; }
  .ci-section-icon-svg { display: flex; align-items: center; color: var(--fg-3); }
  .ci-section-label { font-size: 12px; font-weight: 700; color: var(--fg); flex: 1; }
  .ci-section-count {
    font-size: 11px; font-weight: 700; min-width: 20px; height: 18px;
    padding: 0 5px; border-radius: 99px;
    background: var(--orange-bg); color: var(--orange);
    display: flex; align-items: center; justify-content: center;
  }
  .ci-chevron { color: var(--fg-4); transition: transform 150ms; }
  .ci-chevron.open { transform: rotate(0deg); }
  .ci-chevron:not(.open) { transform: rotate(-90deg); }

  /* Item list */
  .ci-item-list { border-top: 1px solid var(--border); }
  .ci-item {
    display: flex; align-items: center; gap: 0;
    border-bottom: 1px solid var(--border);
    transition: background 120ms;
    position: relative;
  }
  .ci-item:last-child { border-bottom: none; }
  .ci-item:hover { background: var(--surface-2); }

  .ci-item-accent {
    width: 3px; align-self: stretch; flex-shrink: 0; border-radius: 0;
  }

  .ci-item-main {
    flex: 1; padding: 11px 14px; display: flex; flex-direction: column; gap: 5px; min-width: 0;
  }
  .ci-item-top { display: flex; align-items: center; gap: 6px; }
  .ci-priority {
    font-size: 10px; font-weight: 800; padding: 2px 6px;
    border-radius: 99px; flex-shrink: 0; letter-spacing: 0.04em;
  }
  .ci-type-badge {
    font-size: 10px; font-weight: 600; color: var(--fg-4);
    background: var(--surface-2); border: 1px solid var(--border);
    padding: 1px 6px; border-radius: 99px; flex-shrink: 0; text-transform: capitalize;
  }
  .ci-title { font-size: 13px; font-weight: 550; color: var(--fg); line-height: 1.4; }
  .ci-health-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }

  .ci-item-meta { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
  .ci-meta-pill {
    display: flex; align-items: center; gap: 4px;
    font-size: 10px; color: var(--fg-4); background: var(--surface-2);
    border: 1px solid var(--border); border-radius: 99px; padding: 2px 7px;
  }
  .ci-pill-danger { background: rgba(239,68,68,.08); border-color: rgba(239,68,68,.25); color: #ef4444; }
  .ci-pill-warn   { background: rgba(234,179,8,.08); border-color: rgba(234,179,8,.25); color: #ca8a04; }
  .ci-pill-age    { background: transparent; border-color: transparent; color: var(--fg-4); padding: 2px 0; }

  /* Actions */
  .ci-item-actions {
    display: flex; align-items: center; gap: 4px;
    padding: 0 12px; flex-shrink: 0;
  }
  .ci-action-btn {
    display: flex; align-items: center; gap: 4px;
    padding: 5px 9px; border-radius: var(--r); font-size: 11px; font-weight: 600;
    cursor: pointer; transition: all 100ms; border: 1px solid var(--border);
    background: none; color: var(--fg-3); white-space: nowrap;
  }
  .ci-action-ai:hover   { background: var(--orange-bg); border-color: var(--orange); color: var(--orange); }
  .ci-action-done:hover { background: var(--green-bg); border-color: var(--green); color: var(--green); }
  .ci-action-snooze { font-size: 13px; padding: 5px 7px; }
  .ci-action-snooze:hover { background: var(--surface-2); border-color: var(--border-2); }

  .ci-snooze-wrap { position: relative; }
  .ci-snooze-menu {
    position: absolute; right: 0; top: calc(100% + 4px); z-index: 50;
    background: var(--surface); border: 1px solid var(--border);
    border-radius: var(--r); box-shadow: 0 8px 24px rgba(0,0,0,.18);
    display: flex; flex-direction: column; min-width: 90px; overflow: hidden;
  }
  .ci-snooze-menu button {
    padding: 8px 14px; font-size: 11px; font-weight: 500; color: var(--fg-2);
    background: none; border: none; cursor: pointer; text-align: left; transition: background 100ms;
  }
  .ci-snooze-menu button:hover { background: var(--orange-bg); color: var(--orange); }

  /* Footer */
  .ci-footer { font-size: 10px; color: var(--fg-4); text-align: right; padding-top: 4px; }
</style>
