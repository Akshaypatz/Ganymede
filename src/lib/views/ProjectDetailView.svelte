<script lang="ts">
  import { onMount } from "svelte";
  import { activeView, activeProjectId, members, addToast, pendingMeTasks } from "../stores/app";
  import {
    getProject, updateProject, listMembers, createMember, createTask, deleteProject,
    updateTask, deleteTask,
  } from "../api";
  import type { Project, Task, Member, TaskStatus } from "../types";

  const taskStatuses: TaskStatus[] = ["todo", "in_progress", "review", "done"];
  const taskLabels: Record<TaskStatus, string> = {
    todo: "To Do", in_progress: "In Progress", review: "Review", done: "Done",
  };
  const taskColors: Record<TaskStatus, string> = {
    todo: "var(--fg-4)", in_progress: "var(--orange)", review: "#a78bfa", done: "#34d399",
  };

  let project: Project | null = null;
  let loading = true;

  // Task creation
  let newTaskTitle = "";
  let showTaskInput = false;

  // Member creation
  let showAddMember = false;
  let newMemberName = "";
  let newMemberEmail = "";

  // Owner picker
  let showOwnerPicker = false;

  const stages = ["solutioning_pending", "in_development", "released", "live"];
  const stageLabels: Record<string, string> = { solutioning_pending: "Solutioning Pending", in_development: "In Development", released: "Released", live: "Live" };

  onMount(async () => {
    await loadProject();
    const m = await listMembers().catch(() => []);
    members.set(m);
  });

  async function loadProject() {
    if (!$activeProjectId) return;
    loading = true;
    try {
      project = await getProject($activeProjectId);
    } catch (e: any) {
      addToast("Failed to load project", "✗");
    }
    loading = false;
  }

  function daysSince(dateStr: string): number {
    return Math.floor((Date.now() - new Date(dateStr).getTime()) / (1000 * 60 * 60 * 24));
  }

  function fmtDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("en-US", { month: "short", day: "numeric" });
  }

  function goBack() {
    activeView.set("projects");
    activeProjectId.set(null);
  }

  async function setOwner(memberId: string) {
    if (!project) return;
    project = await updateProject({ id: project.id, owner_id: memberId });
    await loadProject();
    showOwnerPicker = false;
    addToast("Owner updated");
  }

  async function setStage(stage: string) {
    if (!project) return;
    project = await updateProject({ id: project.id, stage });
    addToast(`Stage → ${stageLabels[stage]}`);
  }

  async function setHealth(health: string) {
    if (!project) return;
    project = await updateProject({ id: project.id, health });
  }

  let showDeleteConfirm = false;

  async function handleDeleteProject() {
    if (!project) return;
    if (!showDeleteConfirm) { showDeleteConfirm = true; return; }
    showDeleteConfirm = false;
    await deleteProject(project.id);
    addToast(`Deleted project: "${project.name}"`, "✗");
    goBack();
  }

  async function addTask() {
    if (!project || !newTaskTitle.trim()) return;
    const task = await createTask({ project_id: project.id, title: newTaskTitle.trim() });
    project.tasks = [...project.tasks, task];
    newTaskTitle = "";
    showTaskInput = false;
    addToast(`Task added: "${task.title}"`);
    await loadProject();
  }

  async function changeTaskStatus(taskId: string, status: TaskStatus) {
    await updateTask({ id: taskId, status });
    await loadProject();
  }

  async function toggleTaskPendingMe(task: Task) {
    if (task.status === "pending_me") {
      await updateTask({ id: task.id, status: "todo" });
      pendingMeTasks.update(list => list.filter(t => t.id !== task.id));
      addToast(`Unpinned: "${task.title.slice(0, 40)}"`  , "⬡");
    } else {
      await updateTask({ id: task.id, status: "pending_me" });
      const pinned: Task = { ...task, status: "pending_me", project_name: project?.name ?? null };
      pendingMeTasks.update(list => [...list.filter(t => t.id !== task.id), pinned]);
      addToast(`Pinned to Pending on Me`, "⬡");
    }
    await loadProject();
  }

  async function removeTask(taskId: string) {
    await deleteTask(taskId);
    if (project) {
      project.tasks = project.tasks.filter((t) => t.id !== taskId);
    }
    await loadProject();
  }

  async function handleAddMember() {
    if (!newMemberName.trim()) return;
    const member = await createMember({ name: newMemberName.trim(), email: newMemberEmail.trim() || undefined });
    members.update((m) => [...m, member]);
    // Add to project
    if (project) {
      const currentIds = project.members.map((m) => m.id);
      await updateProject({ id: project.id, member_ids: [...currentIds, member.id] });
      await loadProject();
    }
    newMemberName = "";
    newMemberEmail = "";
    showAddMember = false;
    addToast(`Member added: ${member.name}`);
  }

  async function addExistingMember(memberId: string) {
    if (!project) return;
    const currentIds = project.members.map((m) => m.id);
    if (currentIds.includes(memberId)) return;
    await updateProject({ id: project.id, member_ids: [...currentIds, memberId] });
    await loadProject();
    addToast("Member added to project");
  }

  $: tasksByStatus = taskStatuses.reduce((acc, s) => {
    acc[s] = project?.tasks.filter((t) => t.status === s) || [];
    return acc;
  }, {} as Record<TaskStatus, Task[]>);

  $: blockedTasks = project?.tasks.filter((t) => t.status === "blocked") || [];
  $: pendingMeProjectTasks = project?.tasks.filter((t) => t.status === "pending_me") || [];

  $: totalTasks = project?.tasks.length ?? 0;
  $: doneTasks = project?.tasks.filter((t) => t.status === "done").length ?? 0;
  $: computedProgress = totalTasks > 0 ? Math.round((doneTasks / totalTasks) * 100) : 0;

  $: availableMembers = $members.filter(
    (m) => !project?.members.some((pm) => pm.id === m.id)
  );
</script>

{#if loading}
  <div class="loading-state">Loading project…</div>
{:else if project}
  <div class="detail-header">
    <div class="detail-header-top">
      <button class="btn btn-ghost back-btn" on:click={goBack}>← Back</button>
      {#if showDeleteConfirm}
        <div class="delete-confirm-row">
          <span class="delete-confirm-txt">Delete this project?</span>
          <button class="btn-danger-sm" on:click={handleDeleteProject}>Yes, delete</button>
          <button class="btn btn-ghost" style="font-size:11px;padding:4px 10px" on:click={() => showDeleteConfirm = false}>Cancel</button>
        </div>
      {:else}
        <button class="btn btn-danger-ghost" on:click={handleDeleteProject}>Delete Project</button>
      {/if}
    </div>
    <div class="detail-title-row">
      <div>
        <h1 class="detail-name">{project.name}</h1>
        {#if project.description}
          <p class="detail-desc">{project.description}</p>
        {/if}
      </div>
      <div class="health-dot {project.health}" on:click={() => {
        const next = project!.health === 'green' ? 'amber' : project!.health === 'amber' ? 'red' : 'green';
        setHealth(next);
      }} title="Click to cycle health"></div>
    </div>
  </div>

  <!-- Meta Row: Stage, Owner, Progress -->
  <div class="meta-row">
    <div class="meta-block">
      <div class="meta-label">Stage</div>
      <div class="stage-pills">
        {#each stages as s}
          <button
            class="stage-pill"
            class:active={project.stage === s}
            on:click={() => setStage(s)}
          >{stageLabels[s]}</button>
        {/each}
      </div>
    </div>

    <div class="meta-block">
      <div class="meta-label">Owner</div>
      <div class="owner-area">
        {#if project.owner_name}
          <span class="owner-badge" style="--c:{project.members.find(m => m.id === project?.owner_id)?.color || '#f97316'}">{project.owner_name}</span>
        {:else}
          <span class="no-owner">Unassigned</span>
        {/if}
        <button class="btn-icon" on:click={() => showOwnerPicker = !showOwnerPicker} title="Set owner">✎</button>
      </div>
      {#if showOwnerPicker}
        <div class="picker-dropdown">
          {#each $members as m}
            <button class="picker-item" on:click={() => setOwner(m.id)}>
              <span class="avatar-dot" style="background:{m.color}"></span>
              {m.name}
            </button>
          {/each}
          {#if $members.length === 0}
            <div class="picker-empty">No members yet — add one below</div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="meta-block">
      <div class="meta-label">Progress</div>
      <div class="progress-row">
        <div class="prog-bar"><div class="prog-fill" style="width:{computedProgress}%"></div></div>
        <span class="prog-num">{computedProgress}%</span>
      </div>
      <div class="prog-detail">{doneTasks}/{totalTasks} tasks done</div>
    </div>
  </div>

  <!-- Members -->
  <div class="section">
    <div class="section-header">
      <h3>Team ({project.members.length})</h3>
      <button class="btn btn-ghost" on:click={() => showAddMember = !showAddMember}>+ Add</button>
    </div>
    <div class="members-row">
      {#each project.members as m}
        <div class="member-chip" style="--mc:{m.color}">
          <span class="member-avatar" style="background:{m.color}">{m.name[0]}</span>
          <span>{m.name}</span>
        </div>
      {:else}
        <span class="empty-hint">No members assigned</span>
      {/each}
    </div>
    {#if showAddMember}
      <div class="add-member-form">
        {#if availableMembers.length > 0}
          <div class="existing-members">
            <div class="form-label">Existing members:</div>
            {#each availableMembers as m}
              <button class="picker-item" on:click={() => addExistingMember(m.id)}>
                <span class="avatar-dot" style="background:{m.color}"></span>
                {m.name}
              </button>
            {/each}
          </div>
        {/if}
        <div class="new-member-row">
          <input bind:value={newMemberName} placeholder="Name" class="create-input" />
          <input bind:value={newMemberEmail} placeholder="Email (optional)" class="create-input" />
          <button class="btn btn-primary" on:click={handleAddMember}>Add New</button>
        </div>
      </div>
    {/if}
  </div>

  <!-- Tasks -->
  <div class="section">
    <div class="section-header">
      <h3>Tasks ({totalTasks})</h3>
      <button class="btn btn-primary" on:click={() => showTaskInput = !showTaskInput}>+ Add Task</button>
    </div>

    {#if showTaskInput}
      <div class="task-input-row">
        <input
          bind:value={newTaskTitle}
          placeholder="Task title…"
          class="create-input"
          on:keydown={(e) => e.key === "Enter" && addTask()}
          autofocus
        />
        <button class="btn btn-primary" on:click={addTask}>Add</button>
        <button class="btn btn-ghost" on:click={() => showTaskInput = false}>Cancel</button>
      </div>
    {/if}

    {#if blockedTasks.length > 0}
      <div class="blocked-section">
        <div class="blocked-hdr">
          <span class="blocked-icon">⛔</span>
          <span>Blocked ({blockedTasks.length})</span>
        </div>
        {#each blockedTasks as task}
          <div class="blocked-task-row">
            <span class="blocked-task-title">{task.title}</span>
            <button class="btn-micro unblock-btn" on:click={() => changeTaskStatus(task.id, "in_progress")} title="Unblock">→ Unblock</button>
            <button class="btn-micro del" on:click={() => removeTask(task.id)} title="Delete">✗</button>
          </div>
        {/each}
      </div>
    {/if}

    {#if pendingMeProjectTasks.length > 0}
      <div class="pending-me-section">
        <div class="pending-me-hdr">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
          <span>Pending on Me ({pendingMeProjectTasks.length})</span>
        </div>
        {#each pendingMeProjectTasks as task}
          <div class="blocked-task-row">
            <span class="blocked-task-title">{task.title}</span>
            <button class="btn-micro unpin-btn" on:click={() => toggleTaskPendingMe(task)} title="Unpin (back to To Do)">↩ Unpin</button>
            <button class="btn-micro" on:click={() => changeTaskStatus(task.id, "done")} title="Mark done">✓ Done</button>
            <button class="btn-micro del" on:click={() => removeTask(task.id)} title="Delete">✗</button>
          </div>
        {/each}
      </div>
    {/if}

    <div class="tasks-board">
      {#each taskStatuses as status}
        <div class="task-column">
          <div class="task-col-header" style="--col-c:{taskColors[status]}">
            <span class="task-col-dot" style="background:{taskColors[status]}"></span>
            {taskLabels[status]}
            <span class="task-col-count">{tasksByStatus[status].length}</span>
          </div>
          <div class="task-col-list">
            {#each tasksByStatus[status] as task}
              <div class="task-card">
                <div class="task-card-title">{task.title}</div>
                <div class="task-card-meta">
                  {#if task.assignee_name}<span class="task-assignee">{task.assignee_name}</span>{/if}
                  <span class="task-age" class:task-age-old={daysSince(task.created_at) > 7}>
                    {fmtDate(task.created_at)} · {daysSince(task.created_at) < 1 ? 'today' : daysSince(task.created_at) + 'd'}
                  </span>
                </div>
                <div class="task-actions">
                  {#if status !== "done"}
                    <button
                      class="btn-micro"
                      on:click={() => {
                        const idx = taskStatuses.indexOf(status);
                        if (idx < taskStatuses.length - 1) changeTaskStatus(task.id, taskStatuses[idx + 1]);
                      }}
                      title="Move right"
                    >→</button>
                  {/if}
                  {#if status !== "todo"}
                    <button
                      class="btn-micro"
                      on:click={() => {
                        const idx = taskStatuses.indexOf(status);
                        if (idx > 0) changeTaskStatus(task.id, taskStatuses[idx - 1]);
                      }}
                      title="Move left"
                    >←</button>
                  {/if}
                  <button class="btn-micro del" on:click={() => removeTask(task.id)} title="Delete">✗</button>
                  <button class="btn-micro block-btn" on:click={() => changeTaskStatus(task.id, "blocked")} title="Mark blocked">⛔</button>
                  <button class="btn-micro pin-btn" on:click={() => toggleTaskPendingMe(task)} title="Pin to Pending on Me">⬡</button>
                </div>
              </div>
            {:else}
              <div class="task-empty">No tasks</div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>
{:else}
  <div class="empty-state">Project not found. <button class="btn btn-ghost" on:click={goBack}>Go back</button></div>
{/if}

<style>
  .loading-state, .empty-state { text-align: center; padding: 40px 0; color: var(--fg-4); font-size: 13px; }
  .detail-header { margin-bottom: 20px; }
  .detail-header-top { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
  .back-btn { font-size: 12px; }
  .btn-danger-ghost {
    padding: 5px 12px; font-size: 11px; border-radius: var(--r); cursor: pointer; font-weight: 600;
    background: none; border: 1px solid transparent; color: #f87171; transition: all 120ms;
  }
  .btn-danger-ghost:hover { background: #fee2e2; border-color: #ef4444; color: #dc2626; }
  .delete-confirm-row { display: flex; align-items: center; gap: 8px; }
  .delete-confirm-txt { font-size: 11px; color: #f87171; font-weight: 500; }
  .btn-danger-sm {
    padding: 4px 10px; font-size: 11px; font-weight: 700; border-radius: var(--r); cursor: pointer;
    background: #fee2e2; border: 1px solid #ef4444; color: #dc2626;
  }
  .detail-title-row { display: flex; align-items: flex-start; justify-content: space-between; }
  .detail-name { font-size: 22px; font-weight: 800; letter-spacing: -0.04em; margin: 0; }
  .detail-desc { font-size: 12px; color: var(--fg-3); margin: 4px 0 0; }

  .meta-row { display: flex; gap: 24px; margin-bottom: 22px; flex-wrap: wrap; }
  .meta-block { min-width: 160px; }
  .meta-label { font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.08em; color: var(--fg-4); margin-bottom: 6px; }

  .stage-pills { display: flex; gap: 4px; flex-wrap: wrap; }
  .stage-pill {
    padding: 4px 10px; border-radius: 99px; font-size: 10px; font-weight: 600; cursor: pointer;
    background: var(--surface-2); border: 1px solid var(--border); color: var(--fg-3); transition: all 120ms;
  }
  .stage-pill:hover { border-color: var(--border-2); }
  .stage-pill.active { background: var(--orange-bg); color: var(--orange); border-color: var(--orange); }

  .owner-area { display: flex; align-items: center; gap: 6px; }
  .owner-badge {
    display: inline-flex; align-items: center; gap: 4px; padding: 3px 10px; border-radius: 99px;
    font-size: 11px; font-weight: 600; background: var(--surface-2); color: var(--fg);
    border-left: 3px solid var(--c, #f97316);
  }
  .no-owner { font-size: 11px; color: var(--fg-4); }

  .picker-dropdown {
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg);
    padding: 6px; margin-top: 6px; max-height: 160px; overflow-y: auto; box-shadow: var(--shadow);
  }
  .picker-item {
    display: flex; align-items: center; gap: 8px; padding: 6px 8px; border-radius: 6px;
    font-size: 12px; color: var(--fg-2); cursor: pointer; background: none; border: none; width: 100%; text-align: left;
  }
  .picker-item:hover { background: var(--orange-bg); color: var(--orange); }
  .picker-empty { font-size: 11px; color: var(--fg-4); padding: 8px; text-align: center; }
  .avatar-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }

  .progress-row { display: flex; align-items: center; gap: 8px; }
  .prog-bar { flex: 1; background: var(--surface-3); border-radius: 99px; height: 6px; overflow: hidden; min-width: 100px; }
  .prog-fill { height: 100%; border-radius: 99px; background: var(--orange); transition: width 400ms; }
  .prog-num { font-size: 14px; font-weight: 800; font-variant-numeric: tabular-nums; }
  .prog-detail { font-size: 10px; color: var(--fg-4); margin-top: 2px; }

  .section { margin-bottom: 24px; }
  .section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
  .section-header h3 { font-size: 13px; font-weight: 700; margin: 0; }

  .members-row { display: flex; gap: 8px; flex-wrap: wrap; }
  .member-chip {
    display: flex; align-items: center; gap: 6px; padding: 4px 10px 4px 4px;
    border-radius: 99px; background: var(--surface-2); border: 1px solid var(--border); font-size: 11px;
  }
  .member-avatar {
    width: 20px; height: 20px; border-radius: 50%; display: flex; align-items: center; justify-content: center;
    font-size: 9px; font-weight: 700; color: #fff;
  }
  .empty-hint { font-size: 11px; color: var(--fg-4); }

  .add-member-form { margin-top: 8px; }
  .existing-members { margin-bottom: 8px; }
  .form-label { font-size: 10px; font-weight: 600; color: var(--fg-4); margin-bottom: 4px; }
  .new-member-row { display: flex; gap: 6px; align-items: center; }
  .create-input {
    flex: 1; padding: 6px 10px; background: var(--surface-2); border: 1px solid var(--border);
    border-radius: var(--r); font-size: 12px; color: var(--fg);
  }
  .create-input:focus { border-color: var(--orange); outline: none; }

  .task-input-row { display: flex; gap: 6px; align-items: center; margin-bottom: 12px; }

  .tasks-board { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; }
  .task-column { min-height: 80px; }
  .task-col-header {
    display: flex; align-items: center; gap: 6px; padding: 6px 0; margin-bottom: 6px;
    font-size: 11px; font-weight: 700; color: var(--fg-2); border-bottom: 2px solid var(--col-c, var(--border));
  }
  .task-col-dot { width: 7px; height: 7px; border-radius: 50%; }
  .task-col-count {
    margin-left: auto; font-size: 10px; font-weight: 600; color: var(--fg-4);
    background: var(--surface-2); padding: 1px 6px; border-radius: 99px;
  }
  .task-col-list { display: flex; flex-direction: column; gap: 6px; }
  .task-card {
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r);
    padding: 8px 10px; transition: all 120ms;
  }
  .task-card:hover { border-color: var(--border-2); box-shadow: var(--shadow); }
  .task-card-title { font-size: 12px; font-weight: 600; margin-bottom: 4px; }
  .task-card-meta { display: flex; align-items: center; justify-content: space-between; gap: 6px; margin-bottom: 4px; }
  .task-assignee { font-size: 10px; color: var(--fg-4); }
  .task-age { font-size: 9px; color: var(--fg-4); white-space: nowrap; font-variant-numeric: tabular-nums; margin-left: auto; }
  .task-age.task-age-old { color: var(--p0); font-weight: 700; }
  .task-actions { display: flex; gap: 4px; margin-top: 6px; }
  .btn-micro {
    padding: 2px 6px; font-size: 10px; border-radius: 4px; cursor: pointer;
    background: var(--surface-2); border: 1px solid var(--border); color: var(--fg-3);
  }
  .btn-micro:hover { background: var(--orange-bg); color: var(--orange); border-color: var(--orange); }
  .btn-micro.del:hover { background: #fee2e2; color: #ef4444; border-color: #ef4444; }
  .btn-micro.block-btn:hover { background: #fee2e2; color: #ef4444; border-color: #ef4444; }
  .btn-micro.unblock-btn:hover { background: var(--green-bg); color: var(--green); border-color: var(--green); }
  .task-empty { font-size: 11px; color: var(--fg-4); text-align: center; padding: 16px 0; }

  .blocked-section {
    background: rgba(239,68,68,.05); border: 1px solid rgba(239,68,68,.25); border-radius: var(--r-lg);
    padding: 10px 12px; margin-bottom: 14px;
  }
  .blocked-hdr {
    display: flex; align-items: center; gap: 6px; font-size: 11px; font-weight: 700;
    color: #f87171; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.04em;
  }
  .blocked-icon { font-size: 12px; }
  .blocked-task-row {
    display: flex; align-items: center; gap: 8px; padding: 5px 0;
    border-top: 1px solid rgba(239,68,68,.12);
  }
  .blocked-task-title { flex: 1; font-size: 12px; font-weight: 550; color: var(--fg-2); }

  .pending-me-section {
    background: rgba(99,102,241,.05); border: 1px solid rgba(99,102,241,.25); border-radius: var(--r-lg);
    padding: 10px 12px; margin-bottom: 14px;
  }
  .pending-me-hdr {
    display: flex; align-items: center; gap: 6px; font-size: 11px; font-weight: 700;
    color: #818cf8; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.04em;
  }
  .unpin-btn:hover { color: #818cf8; border-color: #818cf8; }
  .pin-btn:hover { color: #818cf8; border-color: rgba(99,102,241,.4); }

  @media (max-width: 900px) {
    .tasks-board { grid-template-columns: repeat(2, 1fr); }
    .meta-row { flex-direction: column; gap: 12px; }
  }
</style>
