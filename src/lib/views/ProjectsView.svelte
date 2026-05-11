<script lang="ts">
  import { onMount } from "svelte";
  import { projects, activeView, activeProjectId, members, addToast } from "../stores/app";
  import { listProjects, createProject, listMembers, deleteProject } from "../api";
  import type { Project } from "../types";

  const stages = ["solutioning_pending", "in_development", "released", "live"];
  const stageLabels: Record<string, string> = { solutioning_pending: "Solutioning Pending", in_development: "In Development", released: "Released", live: "Live" };

  let showCreateForm = false;
  let newName = "";
  let newStage = "solutioning_pending";
  let newDueDate = "";
  let newOwnerId = "";
  let confirmDeleteId: string | null = null;
  let filterStage = "";
  let filterHealth = "";

  $: filteredProjects = $projects.filter((p) => {
    if (filterStage && p.stage !== filterStage) return false;
    if (filterHealth && p.health !== filterHealth) return false;
    return true;
  });

  onMount(async () => {
    try {
      const p = await listProjects();
      projects.set(p);
    } catch {}
    try {
      const m = await listMembers();
      members.set(m);
    } catch {}
  });

  $: stageCounts = stages.reduce((acc, s) => {
    acc[s] = $projects.filter((p) => p.stage === s).length;
    return acc;
  }, {} as Record<string, number>);

  async function handleCreate() {
    if (!newName.trim()) return;
    const proj = await createProject({
      name: newName.trim(),
      stage: newStage,
      due_date: newDueDate || undefined,
      owner_id: newOwnerId || undefined,
    });
    projects.update((list) => [proj, ...list]);
    newName = "";
    newStage = "solutioning_pending";
    newDueDate = "";
    newOwnerId = "";
    showCreateForm = false;
    addToast(`Project created: "${proj.name}"`, "✓");
  }

  function openProject(id: string) {
    activeProjectId.set(id);
    activeView.set("project_detail");
  }

  async function handleDeleteProject(e: Event, project: any) {
    e.stopPropagation();
    if (confirmDeleteId !== project.id) {
      confirmDeleteId = project.id;
      return;
    }
    confirmDeleteId = null;
    await deleteProject(project.id);
    projects.update((list) => list.filter((p) => p.id !== project.id));
    addToast(`Deleted: "${project.name}"`, "✗");
  }
</script>

<div class="page-header">
  <div class="page-title">Projects</div>
  <div class="page-sub">{$projects.length} active · {$projects.filter(p => p.health === 'amber' || p.health === 'red').length} need attention</div>
</div>

<div class="pipeline-wrap">
  {#each stages as stage}
    <div class="pipeline-stage">
      <div class="ps-name">{stageLabels[stage]}</div>
      <div class="ps-num">{stageCounts[stage] || 0}</div>
    </div>
  {/each}
</div>

<div class="toolbar">
  <div class="pill-group">
    <button class="filter-pill" class:active={filterStage === ""} on:click={() => filterStage = ""}>All stages</button>
    {#each stages as s}
      <button class="filter-pill" class:active={filterStage === s} on:click={() => filterStage = filterStage === s ? "" : s}>{stageLabels[s]}</button>
    {/each}
  </div>
  <div class="pill-group">
    <button class="filter-pill health green" class:active={filterHealth === "green"} on:click={() => filterHealth = filterHealth === "green" ? "" : "green"}>On track</button>
    <button class="filter-pill health amber" class:active={filterHealth === "amber"} on:click={() => filterHealth = filterHealth === "amber" ? "" : "amber"}>At risk</button>
    <button class="filter-pill health red" class:active={filterHealth === "red"} on:click={() => filterHealth = filterHealth === "red" ? "" : "red"}>Critical</button>
  </div>
  <div style="flex:1"></div>
  <button class="btn btn-primary" on:click={() => (showCreateForm = !showCreateForm)}>+ New Project</button>
</div>

{#if showCreateForm}
  <div class="create-form">
    <input bind:value={newName} placeholder="Project name…" class="create-input" on:keydown={(e) => e.key === "Enter" && handleCreate()} />
    <select bind:value={newStage} class="create-select">
      {#each stages as s}
        <option value={s}>{stageLabels[s]}</option>
      {/each}
    </select>
    <select bind:value={newOwnerId} class="create-select">
      <option value="">Owner…</option>
      {#each $members as m}
        <option value={m.id}>{m.name}</option>
      {/each}
    </select>
    <input bind:value={newDueDate} type="date" class="create-input small" />
    <button class="btn btn-primary" on:click={handleCreate}>Add</button>
    <button class="btn btn-ghost" on:click={() => (showCreateForm = false)}>Cancel</button>
  </div>
{/if}

<div class="projects-grid">
  {#each filteredProjects as project}
    <div class="proj-card" on:click={() => openProject(project.id)} role="button" tabindex="0" on:keydown={(e) => e.key === 'Enter' && openProject(project.id)}>
      <div class="proj-card-top">
        <div>
          <div class="proj-name">{project.name}</div>
          <div class="proj-stage">{stageLabels[project.stage] || project.stage}</div>
        </div>
        <div class="proj-card-top-right">
          {#if confirmDeleteId === project.id}
            <button class="proj-delete-confirm" on:click={(e) => handleDeleteProject(e, project)}>Sure?</button>
            <button class="proj-delete-cancel" on:click={(e) => { e.stopPropagation(); confirmDeleteId = null; }}>✕</button>
          {:else}
            <button class="proj-delete-btn" on:click={(e) => handleDeleteProject(e, project)} title="Delete project">✕</button>
          {/if}
          <div class="health-dot {project.health}"></div>
        </div>
      </div>
      {#if project.owner_name}
        <div class="proj-owner">{project.owner_name}</div>
      {/if}
      <div class="proj-prog-bar">
        <div class="proj-prog-fill" style="width:{project.progress}%"></div>
      </div>
      <div class="proj-meta">
        <span>{project.progress}%</span>
        {#if project.due_date}
          <span class="proj-meta-pill">Due {new Date(project.due_date).toLocaleDateString("en-US", { month: "short", day: "numeric" })}</span>
        {/if}
      </div>
    </div>
  {:else}
    <div class="empty-state">{filterStage || filterHealth ? 'No projects match the current filters.' : 'No projects yet. Create one to get started.'}</div>
  {/each}
</div>

<style>
  .page-header { margin-bottom: 18px; }
  .page-title { font-size: 17px; font-weight: 750; letter-spacing: -0.03em; }
  .page-sub { font-size: 12px; color: var(--fg-3); margin-top: 3px; }

  .pipeline-wrap {
    display: flex; align-items: stretch; gap: 0;
    border: 1px solid var(--border); border-radius: var(--r-lg); overflow: hidden; margin-bottom: 18px;
  }
  .pipeline-stage {
    flex: 1; padding: 10px 12px; text-align: center;
    border-inline-end: 1px solid var(--border); font-size: 11px;
    cursor: pointer; transition: background 120ms;
  }
  .pipeline-stage:last-child { border-inline-end: none; }
  .pipeline-stage:hover { background: var(--surface-2); }
  .ps-name { font-weight: 600; color: var(--fg-2); margin-bottom: 2px; }
  .ps-num { font-size: 20px; font-weight: 800; font-variant-numeric: tabular-nums; letter-spacing: -0.04em; }

  .toolbar { margin-bottom: 12px; display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .pill-group { display: flex; gap: 4px; flex-wrap: wrap; }
  .filter-pill {
    padding: 4px 10px; border-radius: 99px; font-size: 11px; font-weight: 500;
    border: 1px solid var(--border); background: var(--surface-2); color: var(--fg-3);
    cursor: pointer; transition: all 120ms;
  }
  .filter-pill:hover { border-color: var(--border-2); color: var(--fg); }
  .filter-pill.active { background: var(--surface-3); border-color: var(--border-2); color: var(--fg); }
  .filter-pill.health.green.active { background: var(--green-bg); border-color: var(--green); color: var(--green); }
  .filter-pill.health.amber.active { background: rgba(245,158,11,.1); border-color: #f59e0b; color: #f59e0b; }
  .filter-pill.health.red.active { background: var(--p0-bg); border-color: var(--p0); color: var(--p0); }
  .create-form {
    display: flex; align-items: center; gap: 8px; padding: 10px 12px;
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg); margin-bottom: 14px;
  }
  .create-input {
    flex: 1; padding: 6px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg);
  }
  .create-input.small { flex: 0.3; }
  .create-input:focus { border-color: var(--orange); }
  .create-select {
    padding: 6px 8px; background: var(--surface-2); border: 1px solid var(--border);
    border-radius: var(--r); font-size: 11px; color: var(--fg);
  }

  .projects-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 10px; }
  .proj-card {
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg);
    padding: 14px 16px; cursor: pointer; transition: all 150ms;
  }
  .proj-card:hover { border-color: var(--border-2); box-shadow: var(--shadow); transform: translateY(-1px); }
  .proj-card-top { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 10px; }
  .proj-card-top-right { display: flex; align-items: center; gap: 6px; flex-shrink: 0; }
  .proj-delete-btn {
    width: 20px; height: 20px; border-radius: 4px; border: 1px solid transparent; background: none;
    color: var(--fg-4); font-size: 10px; cursor: pointer; display: flex; align-items: center; justify-content: center;
    transition: all 100ms; opacity: 0;
  }
  .proj-card:hover .proj-delete-btn { opacity: 1; }
  .proj-delete-btn:hover { background: #fee2e2; border-color: #ef4444; color: #ef4444; }
  .proj-delete-confirm {
    font-size: 9px; font-weight: 700; padding: 2px 7px; border-radius: 4px; cursor: pointer;
    background: #fee2e2; border: 1px solid #ef4444; color: #ef4444;
  }
  .proj-delete-cancel {
    width: 18px; height: 18px; border-radius: 4px; border: 1px solid var(--border); background: none;
    color: var(--fg-4); font-size: 9px; cursor: pointer; display: flex; align-items: center; justify-content: center;
  }
  .proj-name { font-size: 13px; font-weight: 650; color: var(--fg); }
  .proj-stage { font-size: 10px; color: var(--fg-4); margin-top: 2px; }
  .proj-owner { font-size: 10px; color: var(--fg-3); margin-bottom: 6px; }
  .proj-prog-bar { background: var(--surface-3); border-radius: 99px; height: 4px; overflow: hidden; margin-bottom: 8px; }
  .proj-prog-fill { height: 100%; border-radius: 99px; background: var(--orange); transition: width 500ms; }
  .proj-meta { display: flex; align-items: center; gap: 10px; font-size: 10px; color: var(--fg-4); }
  .proj-meta-pill {
    display: flex; align-items: center; gap: 4px;
    padding: 2px 7px; border-radius: 99px; border: 1px solid var(--border-2); color: var(--fg-3);
  }
  .empty-state { font-size: 12px; color: var(--fg-4); padding: 24px 0; text-align: center; grid-column: 1 / -1; }
</style>
