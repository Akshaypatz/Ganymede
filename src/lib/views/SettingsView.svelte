<script lang="ts">
  import { onMount } from "svelte";
  import {
    addToast,
    members,
    followupCategories,
    attentionRules,
    projects,
    items,
    aiConversationId,
    aiMessages,
  } from "../stores/app";
  import {
    getSetting,
    setSetting,
    listMembers,
    listItems,
    createMember,
    deleteMember,
    listProjects,
    createProject,
    createItem,
    createTask,
    resetAppData,
  } from "../api";
  import { FOLLOWUP_TAGS } from "../types";

  let activeTab: "ai" | "team" | "config" | "data" = "ai";

  let aiProvider = "openai";
  let aiApiKey = "";
  let aiModel = "";
  let aiEndpoint = "";
  let loading = true;

  // Member management
  let newMemberName = "";
  let newMemberEmail = "";

  const memberColors = ["#f97316", "#3b82f6", "#8b5cf6", "#ef4444", "#10b981", "#f59e0b", "#ec4899", "#06b6d4"];
  let colorIdx = 0;

  // Provider presets
  const providerPresets: Record<string, { endpoint: string; model: string; keyHint: string; keyUrl: string; free?: boolean }> = {
    openai: {
      endpoint: "https://api.openai.com",
      model: "gpt-4o-mini",
      keyHint: "sk-…",
      keyUrl: "https://platform.openai.com/api-keys",
    },
    openrouter: {
      endpoint: "https://openrouter.ai/api",
      model: "meta-llama/llama-3.1-8b-instruct:free",
      keyHint: "sk-or-…",
      keyUrl: "https://openrouter.ai/keys",
      free: true,
    },
    groq: {
      endpoint: "https://api.groq.com/openai",
      model: "llama-3.3-70b-versatile",
      keyHint: "gsk_…",
      keyUrl: "https://console.groq.com/keys",
      free: true,
    },
    github_copilot: {
      endpoint: "https://models.inference.ai.azure.com",
      model: "gpt-4o-mini",
      keyHint: "GitHub PAT (ghp_…)",
      keyUrl: "https://github.com/settings/tokens",
      free: true,
    },
    ollama: {
      endpoint: "http://localhost:11434",
      model: "llama3.1",
      keyHint: "(no key needed)",
      keyUrl: "",
      free: true,
    },
  };

  // Config tab
  let newCatLabel = "";
  let newCatColor = "#6366f1";
  let rules = { p0: true, blocked: true, stale_p1_days: 7, overdue_followups: true };
  $: rules = { ...$attentionRules };

  // Data tab: Excel import
  let importFileName = "";
  let importRows: Record<string, any>[] = [];
  let importHeaders: string[] = [];
  let importSummary = { projects: 0, items: 0, tasks: 0, skipped: 0 };
  let importing = false;
  let importBusy = false;
  let columnMapping: Record<string, string> = {}; // header → field key
  let showMappingEditor = false;

  const MAPPING_FIELDS = [
    { value: "skip",          label: "— skip —" },
    { value: "title",         label: "Title / Issue name" },
    { value: "body",          label: "Description / Body" },
    { value: "project",       label: "Project name" },
    { value: "project_stage", label: "Project stage" },
    { value: "status",        label: "Status" },
    { value: "priority",      label: "Priority" },
    { value: "assignee",      label: "Assignee" },
    { value: "due_at",        label: "Due date" },
    { value: "type",          label: "Item type" },
    { value: "task",          label: "Task title" },
  ];

  // Data tab: reset
  let resetConfirmText = "";
  let resetting = false;

  function onProviderChange() {
    const preset = providerPresets[aiProvider];
    if (preset) {
      if (!aiEndpoint || Object.values(providerPresets).some((p) => p.endpoint === aiEndpoint)) {
        aiEndpoint = preset.endpoint;
      }
      if (!aiModel || Object.values(providerPresets).some((p) => p.model === aiModel)) {
        aiModel = preset.model;
      }
    }
  }

  function normalize(text: string): string {
    return (text || "").toLowerCase().replace(/[^a-z0-9]+/g, "").trim();
  }

  function pickHeader(headers: string[], candidates: string[]): string | null {
    const candidateNorms = candidates.map(normalize);
    for (const h of headers) {
      const n = normalize(h);
      if (candidateNorms.includes(n)) return h;
    }
    for (const h of headers) {
      const n = normalize(h);
      if (candidateNorms.some((c) => n.includes(c) || c.includes(n))) return h;
    }
    return null;
  }

  function mapPriority(raw: string): "p0" | "p1" | "p2" | "p3" {
    const v = (raw || "").toLowerCase();
    if (v.includes("p0") || v.includes("sev0") || v.includes("critical") || v.includes("blocker")) return "p0";
    if (v.includes("p1") || v.includes("sev1") || v.includes("high")) return "p1";
    if (v.includes("p3") || v.includes("sev3") || v.includes("low")) return "p3";
    return "p2";
  }

  function mapProjectStage(raw: string): "solutioning_pending" | "in_development" | "released" | "live" {
    const v = (raw || "").toLowerCase();
    if (v.includes("live") || v.includes("prod")) return "live";
    if (v.includes("released") || v.includes("release") || v.includes("qa")) return "released";
    if (v.includes("develop") || v.includes("build") || v.includes("impl")) return "in_development";
    return "solutioning_pending";
  }

  function mapItemType(raw: string): "issue" | "followup" | "decision" | "idea" | "initiative" | "note" | "waiting" {
    const v = (raw || "").toLowerCase();
    if (v.includes("follow")) return "followup";
    if (v.includes("decision")) return "decision";
    if (v.includes("idea")) return "idea";
    if (v.includes("initiative")) return "initiative";
    if (v.includes("waiting")) return "waiting";
    if (v.includes("note")) return "note";
    return "issue";
  }

  function mapItemStatus(raw: string, itemType: string): string {
    const v = (raw || "").toLowerCase();
    if (itemType === "idea") {
      if (v.includes("explor")) return "exploring";
      if (v.includes("plan")) return "planned";
      if (v.includes("ready")) return "ready";
      if (v.includes("done") || v.includes("archive")) return "done";
      return "brainstorm";
    }
    if (v.includes("done") || v.includes("closed") || v.includes("resolved")) return "done";
    if (v.includes("block")) return "blocked";
    if (v.includes("progress") || v.includes("wip") || v.includes("doing")) return "in_progress";
    if (v.includes("wait")) return "waiting";
    if (v.includes("snooz")) return "snoozed";
    return "open";
  }

  function mapTaskStatus(raw: string): "todo" | "in_progress" | "review" | "blocked" | "done" {
    const v = (raw || "").toLowerCase();
    if (v.includes("done") || v.includes("closed")) return "done";
    if (v.includes("block")) return "blocked";
    if (v.includes("review") || v.includes("qa")) return "review";
    if (v.includes("progress") || v.includes("doing") || v.includes("wip")) return "in_progress";
    return "todo";
  }

  async function handleExcelSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    importBusy = true;
    showMappingEditor = false;
    try {
      importFileName = file.name;
      const XLSX = await import("xlsx");
      const buf = await file.arrayBuffer();
      const wb = XLSX.read(buf, { type: "array" });
      const firstSheet = wb.SheetNames[0];
      if (!firstSheet) throw new Error("No sheet found in file");
      const ws = wb.Sheets[firstSheet];
      const rows = XLSX.utils.sheet_to_json<Record<string, any>>(ws, { defval: "" });
      importRows = rows;
      importHeaders = rows.length ? Object.keys(rows[0]) : [];

      // Auto-detect column mapping
      const fieldCandidates: Record<string, string[]> = {
        task:          ["task", "task_title", "subtask", "action_item", "action"],
        title:         ["title", "issue", "issue_title", "summary", "item", "name"],
        body:          ["description", "details", "body", "notes", "comment"],
        project:       ["project", "project_name", "workstream", "stream"],
        project_stage: ["project_stage", "stage", "phase"],
        status:        ["status", "state"],
        priority:      ["priority", "severity"],
        assignee:      ["assignee", "owner", "engineer", "person"],
        due_at:        ["due", "due_date", "deadline", "eta", "target_date"],
        type:          ["type", "category", "kind"],
      };

      const newMapping: Record<string, string> = {};
      const usedFields = new Set<string>();
      for (const header of importHeaders) {
        let matched = "skip";
        for (const [field, candidates] of Object.entries(fieldCandidates)) {
          if (usedFields.has(field)) continue;
          const hn = normalize(header);
          if (candidates.some(c => normalize(c) === hn || hn.includes(normalize(c)) || normalize(c).includes(hn))) {
            matched = field;
            usedFields.add(field);
            break;
          }
        }
        newMapping[header] = matched;
      }
      columnMapping = newMapping;

      // Count preview using detected mapping
      recomputePreview();
      showMappingEditor = true;
      addToast(`Parsed ${rows.length} rows from ${file.name}`, "✓");
    } catch (err: any) {
      importRows = [];
      importHeaders = [];
      importSummary = { projects: 0, items: 0, tasks: 0, skipped: 0 };
      addToast(`Import parse failed: ${err?.message || err}`, "✗");
    } finally {
      importBusy = false;
    }
  }

  function recomputePreview() {
    const projectsSet = new Set<string>();
    let items = 0, tasks = 0, skipped = 0;
    const projectHeader = Object.keys(columnMapping).find(h => columnMapping[h] === "project");
    const taskHeader    = Object.keys(columnMapping).find(h => columnMapping[h] === "task");
    const titleHeader   = Object.keys(columnMapping).find(h => columnMapping[h] === "title");
    for (const row of importRows) {
      const pname = String(projectHeader ? row[projectHeader] : "").trim();
      if (pname) projectsSet.add(pname.toLowerCase());
      const taskTitle = String(taskHeader ? row[taskHeader] : "").trim();
      const itemTitle = String(titleHeader ? row[titleHeader] : "").trim();
      if (taskTitle) tasks++;
      else if (itemTitle) items++;
      else skipped++;
    }
    importSummary = { projects: projectsSet.size, items, tasks, skipped };
  }

  $: if (columnMapping && importRows.length) recomputePreview();

  async function runExcelImport() {
    if (!importRows.length) return;
    importing = true;
    try {
      // Use the column mapping to find the right header for each field
      function mappedHeader(field: string): string | null {
        const h = Object.keys(columnMapping).find(k => columnMapping[k] === field);
        return h ?? null;
      }

      const projectHeader      = mappedHeader("project");
      const projectStageHeader = mappedHeader("project_stage");
      const titleHeader        = mappedHeader("title");
      const taskHeader         = mappedHeader("task");
      const descHeader         = mappedHeader("body");
      const priorityHeader     = mappedHeader("priority");
      const statusHeader       = mappedHeader("status");
      const assigneeHeader     = mappedHeader("assignee");
      const dueHeader          = mappedHeader("due_at");
      const typeHeader         = mappedHeader("type");

      const existingProjects = await listProjects().catch(() => []);
      const projectMap = new Map(existingProjects.map((p) => [p.name.toLowerCase(), p]));
      const createdProjects = new Set<string>();

      async function ensureProject(name: string, stageText: string, dueText: string) {
        const key = name.toLowerCase();
        if (projectMap.has(key)) return projectMap.get(key)!;
        const created = await createProject({
          name,
          stage: mapProjectStage(stageText),
          due_date: dueText || undefined,
        });
        projectMap.set(key, created);
        createdProjects.add(key);
        return created;
      }

      let importedItems = 0;
      let importedTasks = 0;

      for (const row of importRows) {
        const projectName = String(projectHeader ? row[projectHeader] : "").trim();
        const stageText = String(projectStageHeader ? row[projectStageHeader] : "").trim();
        const taskTitle = String(taskHeader ? row[taskHeader] : "").trim();
        const itemTitle = String(titleHeader ? row[titleHeader] : "").trim();
        const descText = String(descHeader ? row[descHeader] : "").trim();
        const priorityText = String(priorityHeader ? row[priorityHeader] : "").trim();
        const statusText = String(statusHeader ? row[statusHeader] : "").trim();
        const assigneeText = String(assigneeHeader ? row[assigneeHeader] : "").trim();
        const dueText = String(dueHeader ? row[dueHeader] : "").trim();
        const typeText = String(typeHeader ? row[typeHeader] : "").trim();

        if (taskTitle) {
          const effectiveProject = projectName || "Imported Project";
          const p = await ensureProject(effectiveProject, stageText, dueText);
          await createTask({
            project_id: p.id,
            title: taskTitle,
            description: descText || undefined,
            status: mapTaskStatus(statusText),
          });
          importedTasks += 1;
          continue;
        }

        if (itemTitle) {
          const itemType = mapItemType(typeText);
          let projectId: string | undefined;
          if (projectName) {
            const p = await ensureProject(projectName, stageText, dueText);
            projectId = p.id;
          }
          await createItem({
            title: itemTitle,
            body: descText || undefined,
            type: itemType,
            status: mapItemStatus(statusText, itemType),
            priority: mapPriority(priorityText),
            assignee: assigneeText || undefined,
            due_at: dueText || undefined,
            project_id: projectId,
          } as any);
          importedItems += 1;
        }
      }

      const [p, it, m] = await Promise.all([
        listProjects().catch(() => []),
        listItems().catch(() => []),
        listMembers().catch(() => []),
      ]);
      projects.set(p);
      items.set(it);
      members.set(m);

      addToast(
        `Imported ${importedItems} items, ${importedTasks} tasks, ${createdProjects.size} new projects`,
        "✓"
      );
    } catch (err: any) {
      addToast(`Import failed: ${err?.message || err}`, "✗");
    } finally {
      importing = false;
    }
  }

  async function performReset() {
    if (resetConfirmText.trim().toUpperCase() !== "RESET") {
      addToast("Type RESET to confirm", "⚠");
      return;
    }
    resetting = true;
    try {
      await resetAppData();
      projects.set([]);
      items.set([]);
      members.set([]);
      aiConversationId.set(null);
      aiMessages.set([]);
      followupCategories.set([...FOLLOWUP_TAGS]);
      attentionRules.set({ p0: true, blocked: true, stale_p1_days: 7, overdue_followups: true });
      resetConfirmText = "";
      addToast("App data reset complete", "✓");
    } catch (err: any) {
      addToast(`Reset failed: ${err?.message || err}`, "✗");
    } finally {
      resetting = false;
    }
  }

  async function saveAiSettings() {
    await setSetting("ai_provider", aiProvider);
    await setSetting("ai_api_key", aiApiKey);
    await setSetting("ai_model", aiModel || providerPresets[aiProvider]?.model || "");
    await setSetting("ai_endpoint", aiEndpoint || providerPresets[aiProvider]?.endpoint || "");
    addToast("AI settings saved", "✓");
  }

  async function handleAddMember() {
    if (!newMemberName.trim()) return;
    const color = memberColors[colorIdx % memberColors.length];
    colorIdx += 1;
    const member = await createMember({
      name: newMemberName.trim(),
      email: newMemberEmail.trim() || undefined,
      color,
    });
    members.update((m) => [...m, member]);
    newMemberName = "";
    newMemberEmail = "";
    addToast(`Member added: ${member.name}`);
  }

  async function handleDeleteMember(id: string) {
    await deleteMember(id);
    members.update((m) => m.filter((x) => x.id !== id));
    addToast("Member removed");
  }

  async function saveAttentionRules() {
    attentionRules.set({ ...rules });
    await setSetting("attention_rules", JSON.stringify(rules));
    addToast("Attention rules saved", "✓");
  }

  async function addCategory() {
    const label = newCatLabel.trim();
    if (!label) return;
    const value = label.toLowerCase().replace(/\s+/g, "_");
    const updated = [...$followupCategories, { value, label, color: newCatColor }];
    followupCategories.set(updated);
    await setSetting("followup_categories", JSON.stringify(updated));
    newCatLabel = "";
    newCatColor = "#6366f1";
    addToast(`Category added: ${label}`, "✓");
  }

  async function removeCategory(value: string) {
    const updated = $followupCategories.filter((c) => c.value !== value);
    followupCategories.set(updated);
    await setSetting("followup_categories", JSON.stringify(updated));
    addToast("Category removed");
  }

  $: preset = providerPresets[aiProvider];

  onMount(async () => {
    aiProvider = (await getSetting("ai_provider").catch(() => null)) || "github_copilot";
    aiApiKey = (await getSetting("ai_api_key").catch(() => null)) || "";
    aiModel = (await getSetting("ai_model").catch(() => null)) || "";
    aiEndpoint = (await getSetting("ai_endpoint").catch(() => null)) || "";
    const m = await listMembers().catch(() => []);
    members.set(m);

    try {
      const saved = await getSetting("attention_rules").catch(() => null);
      if (saved) attentionRules.set(JSON.parse(saved));
    } catch {}

    try {
      const saved = await getSetting("followup_categories").catch(() => null);
      if (saved) {
        const parsed = JSON.parse(saved);
        if (Array.isArray(parsed) && parsed.length) followupCategories.set(parsed);
      }
    } catch {}

    loading = false;
  });
</script>

<div class="page-header">
  <div class="page-title">Settings</div>
  <div class="page-sub">Configure your workspace</div>
</div>

<div class="tabs-bar">
  <button class="tab" class:active={activeTab === "ai"} on:click={() => activeTab = "ai"}>AI Assistant</button>
  <button class="tab" class:active={activeTab === "team"} on:click={() => activeTab = "team"}>Team</button>
  <button class="tab" class:active={activeTab === "config"} on:click={() => activeTab = "config"}>Config</button>
  <button class="tab" class:active={activeTab === "data"} on:click={() => activeTab = "data"}>Data</button>
</div>

{#if !loading}
  {#if activeTab === "ai"}
  <div class="settings-section">
    <h3 class="section-title">AI Assistant</h3>

    <div class="setting-row">
      <label class="setting-label">Provider</label>
      <select bind:value={aiProvider} class="setting-select" on:change={onProviderChange}>
        <option value="openai">OpenAI</option>
        <option value="openrouter">OpenRouter (free models available)</option>
        <option value="groq">Groq (free tier)</option>
        <option value="github_copilot">GitHub Copilot</option>
        <option value="ollama">Ollama (local)</option>
      </select>
    </div>

    {#if preset?.free}
      <div class="provider-badge">
        ✓ Free tier available
        {#if preset.keyUrl}
          · <a href={preset.keyUrl} target="_blank" rel="noopener noreferrer">Get API key →</a>
        {/if}
      </div>
    {/if}

    {#if aiProvider === "openrouter"}
      <div class="provider-info">
        OpenRouter gives access to 50+ models. Free models end in <code>:free</code> — e.g. <code>meta-llama/llama-3.1-8b-instruct:free</code>, <code>google/gemma-2-9b-it:free</code>, <code>mistralai/mistral-7b-instruct:free</code>
      </div>
    {:else if aiProvider === "groq"}
      <div class="provider-info">
        Groq provides ultra-fast inference. Free tier models: <code>llama-3.3-70b-versatile</code>, <code>llama-3.1-8b-instant</code>, <code>mixtral-8x7b-32768</code>
      </div>
    {:else if aiProvider === "github_copilot"}
      <div class="provider-info">
        Uses your GitHub Copilot subscription. Create a Personal Access Token at github.com/settings/tokens — select <code>models:read</code> scope (under "Models"). Works with <code>gpt-4o-mini</code>, <code>gpt-4o</code>, <code>claude-3.5-sonnet</code>.
      </div>
    {:else if aiProvider === "ollama"}
      <div class="provider-info">
        Runs fully local — no API key needed. Make sure Ollama is running: <code>ollama serve</code>
      </div>
    {/if}

    {#if aiProvider !== "ollama"}
      <div class="setting-row">
        <label class="setting-label">API Key</label>
        <input type="password" bind:value={aiApiKey} class="setting-input" placeholder={preset?.keyHint || "API key"} />
      </div>
    {/if}

    <div class="setting-row">
      <label class="setting-label">Model</label>
      <input bind:value={aiModel} class="setting-input" placeholder={preset?.model || "model name"} />
    </div>
    <div class="setting-row">
      <label class="setting-label">Endpoint</label>
      <input bind:value={aiEndpoint} class="setting-input" placeholder={preset?.endpoint || "https://..."} />
    </div>

    <button class="btn btn-primary" on:click={saveAiSettings}>Save AI Settings</button>
  </div>
  {/if}

  {#if activeTab === "team"}
  <div class="settings-section">
    <h3 class="section-title">Team Directory</h3>
    <p class="section-desc">Members added here are available as project owners, task assignees, and are remembered for future use.</p>

    <div class="members-list">
      {#each $members as m}
        <div class="member-row">
          <span class="member-avatar" style="background:{m.color}">{m.name[0]}</span>
          <div class="member-info">
            <div class="member-name">{m.name}</div>
            {#if m.email}<div class="member-email">{m.email}</div>{/if}
          </div>
          <button class="btn-icon del-btn" on:click={() => handleDeleteMember(m.id)} title="Remove">✗</button>
        </div>
      {:else}
        <div class="empty-hint">No team members yet</div>
      {/each}
    </div>

    <div class="add-member-row">
      <input bind:value={newMemberName} placeholder="Name" class="setting-input" on:keydown={(e) => e.key === 'Enter' && handleAddMember()} />
      <input bind:value={newMemberEmail} placeholder="Email (optional)" class="setting-input" />
      <button class="btn btn-primary" on:click={handleAddMember}>Add</button>
    </div>
  </div>
  {/if}

  {#if activeTab === "config"}
  <div class="settings-section">
    <h3 class="section-title">Follow-up Categories</h3>
    <p class="section-desc">Define the categories used to tag follow-up items. These appear as filters in the Follow-ups view.</p>
    <div class="cat-list">
      {#each $followupCategories as cat}
        <div class="cat-row">
          <span class="cat-swatch" style="background:{cat.color}"></span>
          <span class="cat-label">{cat.label}</span>
          <button class="btn-icon del-btn" on:click={() => removeCategory(cat.value)} title="Remove">✗</button>
        </div>
      {/each}
    </div>
    <div class="add-cat-row">
      <input bind:value={newCatLabel} placeholder="Category name" class="setting-input" on:keydown={(e) => e.key === 'Enter' && addCategory()} style="max-width:200px" />
      <label class="color-pick-label">
        <input type="color" bind:value={newCatColor} class="color-pick" />
        <span class="color-pick-swatch" style="background:{newCatColor}"></span>
      </label>
      <button class="btn btn-primary" on:click={addCategory}>Add Category</button>
    </div>
  </div>

  <div class="settings-section">
    <h3 class="section-title">Needs Attention Rules</h3>
    <p class="section-desc">Control what appears in the "Needs Attention" section on the Dashboard.</p>
    <div class="rule-list">
      <label class="rule-row">
        <input type="checkbox" bind:checked={rules.p0} />
        <div class="rule-info">
          <span class="rule-name">P0 Critical issues</span>
          <span class="rule-desc">Any issue with P0 priority that is not done</span>
        </div>
      </label>
      <label class="rule-row">
        <input type="checkbox" bind:checked={rules.blocked} />
        <div class="rule-info">
          <span class="rule-name">Blocked issues</span>
          <span class="rule-desc">Any issue with status "Blocked"</span>
        </div>
      </label>
      <label class="rule-row">
        <input type="checkbox" bind:checked={rules.overdue_followups} />
        <div class="rule-info">
          <span class="rule-name">Overdue follow-ups</span>
          <span class="rule-desc">Follow-ups whose due date has passed</span>
        </div>
      </label>
      <label class="rule-row">
        <input type="checkbox" bind:checked={rules.p0} />
        <div class="rule-info">
          <span class="rule-name">Stale P1 issues</span>
          <span class="rule-desc">P1 issues open longer than
            <input type="number" bind:value={rules.stale_p1_days} min="1" max="30" class="days-input" /> days</span>
        </div>
      </label>
    </div>
    <button class="btn btn-primary" on:click={saveAttentionRules}>Save Rules</button>
  </div>
  {/if}

  {#if activeTab === "data"}
  <div class="settings-section">
    <h3 class="section-title">Excel Import</h3>
    <p class="section-desc">Upload your old workbook. Ganymede will auto-detect columns — you can adjust the mapping before importing.</p>

    <div class="import-row">
      <label class="upload-btn">
        <input type="file" accept=".xlsx,.xls,.csv" on:change={handleExcelSelect} class="file-input-hidden" />
        {importBusy ? "Parsing…" : importFileName ? "Change file" : "Choose file…"}
      </label>
      {#if importFileName}
        <span class="file-name">{importFileName}</span>
      {/if}
    </div>

    {#if showMappingEditor && importHeaders.length > 0}
      <div class="mapping-editor">
        <div class="mapping-header-row">
          <span class="mapping-col-label">Spreadsheet column</span>
          <span class="mapping-col-label">Import as</span>
        </div>
        {#each importHeaders as header}
          <div class="mapping-row">
            <span class="mapping-col-name">{header}</span>
            <select
              class="mapping-select"
              bind:value={columnMapping[header]}
              on:change={() => columnMapping = { ...columnMapping }}
            >
              {#each MAPPING_FIELDS as f}
                <option value={f.value}>{f.label}</option>
              {/each}
            </select>
          </div>
        {/each}
      </div>

      <div class="import-summary">
        <div><strong>{importRows.length}</strong> rows total</div>
        <div><strong>{importSummary.projects}</strong> unique projects</div>
        <div><strong>{importSummary.items}</strong> items</div>
        <div><strong>{importSummary.tasks}</strong> tasks</div>
        {#if importSummary.skipped > 0}
          <div class="skip-count"><strong>{importSummary.skipped}</strong> skipped (no title)</div>
        {/if}
      </div>

      <button class="btn btn-primary" on:click={runExcelImport} disabled={importing || importBusy}>
        {importing ? "Importing…" : "Import Into Ganymede"}
      </button>
    {/if}
  </div>

  <div class="settings-section danger-zone">
    <h3 class="section-title">Reset App Data</h3>
    <p class="section-desc">This wipes all user-entered data from SQLite: projects, items, tasks, members, boards, AI history, settings, and activity.</p>
    <div class="reset-row">
      <input
        bind:value={resetConfirmText}
        class="setting-input"
        placeholder="Type RESET to confirm"
        style="max-width:240px"
      />
      <button class="btn btn-danger" on:click={performReset} disabled={resetting}>
        {resetting ? "Resetting..." : "Reset Everything"}
      </button>
    </div>
  </div>
  {/if}
{/if}

<style>
  .page-header { margin-bottom: 22px; }
  .page-title { font-size: 17px; font-weight: 750; letter-spacing: -0.03em; }
  .page-sub { font-size: 12px; color: var(--fg-3); margin-top: 3px; }

  .settings-section {
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg);
    padding: 18px 20px; margin-bottom: 16px;
  }
  .section-title { font-size: 13px; font-weight: 700; margin: 0 0 12px; }
  .section-desc { font-size: 11px; color: var(--fg-3); margin: 0 0 12px; }

  .setting-row { display: flex; align-items: center; gap: 12px; margin-bottom: 10px; }
  .setting-label { font-size: 11px; font-weight: 600; color: var(--fg-3); min-width: 70px; }
  .setting-input {
    flex: 1; padding: 7px 10px; background: var(--surface-2); border: 1px solid var(--border);
    border-radius: var(--r); font-size: 12px; color: var(--fg); max-width: 400px;
  }
  .setting-input:focus { border-color: var(--orange); outline: none; }
  .setting-select {
    padding: 7px 10px; background: var(--surface-2); border: 1px solid var(--border);
    border-radius: var(--r); font-size: 12px; color: var(--fg);
  }

  .provider-badge {
    font-size: 11px; color: #10b981; background: rgba(16,185,129,.1);
    border: 1px solid rgba(16,185,129,.25); border-radius: var(--r);
    padding: 5px 10px; margin-bottom: 10px; display: inline-flex; gap: 4px; align-items: center;
  }
  .provider-badge a { color: #10b981; text-decoration: underline; }
  .provider-info {
    font-size: 11px; color: var(--fg-3); background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r);
    padding: 8px 10px; margin-bottom: 10px; line-height: 1.5;
  }
  .provider-info :global(code) {
    background: var(--surface-3); padding: 1px 4px; border-radius: 3px;
    font-size: 10px; color: var(--orange);
  }

  .members-list { margin-bottom: 12px; }
  .member-row {
    display: flex; align-items: center; gap: 10px; padding: 7px 0;
    border-bottom: 1px solid var(--border);
  }
  .member-row:last-child { border-bottom: none; }
  .member-avatar {
    width: 26px; height: 26px; border-radius: 50%; display: flex; align-items: center; justify-content: center;
    font-size: 11px; font-weight: 700; color: #fff; flex-shrink: 0;
  }
  .member-info { flex: 1; }
  .member-name { font-size: 12px; font-weight: 600; }
  .member-email { font-size: 10px; color: var(--fg-4); }
  .del-btn { color: var(--fg-4); font-size: 12px; }
  .del-btn:hover { color: #ef4444; }
  .empty-hint { font-size: 11px; color: var(--fg-4); padding: 8px 0; }

  .add-member-row { display: flex; gap: 6px; align-items: center; }

  /* Tabs */
  .tabs-bar {
    display: flex; gap: 2px; margin-bottom: 16px;
    background: var(--surface); border: 1px solid var(--border);
    border-radius: var(--r-lg); padding: 4px; width: fit-content;
  }
  .tab {
    padding: 6px 16px; border-radius: var(--r); font-size: 12px; font-weight: 500;
    color: var(--fg-3); background: none; border: none; cursor: pointer; transition: all 120ms;
  }
  .tab:hover { color: var(--fg); }
  .tab.active { background: var(--surface-2); color: var(--fg); border: 1px solid var(--border); }

  /* Config tab */
  .cat-list { margin-bottom: 12px; }
  .cat-row {
    display: flex; align-items: center; gap: 10px; padding: 7px 0;
    border-bottom: 1px solid var(--border);
  }
  .cat-row:last-child { border-bottom: none; }
  .cat-swatch { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
  .cat-label { flex: 1; font-size: 12px; font-weight: 500; }
  .add-cat-row { display: flex; align-items: center; gap: 8px; }
  .color-pick-label { position: relative; display: flex; align-items: center; cursor: pointer; }
  .color-pick { position: absolute; opacity: 0; width: 0; height: 0; }
  .color-pick-swatch { width: 28px; height: 28px; border-radius: var(--r); border: 1px solid var(--border); cursor: pointer; }

  .rule-list { margin-bottom: 16px; display: flex; flex-direction: column; gap: 1px; }
  .rule-row {
    display: flex; align-items: flex-start; gap: 12px; padding: 10px 12px;
    background: var(--surface-2); border-radius: var(--r); cursor: pointer;
    transition: background 100ms; border: 1px solid transparent;
  }
  .rule-row:hover { background: var(--surface-3); }
  .rule-row input[type="checkbox"] { margin-top: 2px; accent-color: var(--orange); flex-shrink: 0; }
  .rule-info { display: flex; flex-direction: column; gap: 2px; }
  .rule-name { font-size: 12px; font-weight: 600; }
  .rule-desc { font-size: 11px; color: var(--fg-3); display: flex; align-items: center; gap: 4px; }
  .days-input {
    width: 42px; padding: 1px 4px; background: var(--surface); border: 1px solid var(--border);
    border-radius: 4px; font-size: 11px; color: var(--fg); text-align: center;
  }

  .import-row { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }
  .upload-btn {
    display: inline-flex; align-items: center; padding: 6px 12px;
    background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--r);
    font-size: 12px; font-weight: 600; color: var(--fg-3); cursor: pointer;
    transition: border-color .15s;
  }
  .upload-btn:hover { border-color: var(--orange); color: var(--fg); }
  .file-input-hidden { display: none; }
  .file-name { font-size: 11px; color: var(--fg-4); }

  .mapping-editor {
    border: 1px solid var(--border); border-radius: var(--r-lg);
    overflow: hidden; margin-bottom: 12px;
  }
  .mapping-header-row {
    display: grid; grid-template-columns: 1fr 1fr;
    background: var(--surface-2); padding: 7px 12px;
    border-bottom: 1px solid var(--border);
  }
  .mapping-col-label { font-size: 10px; font-weight: 700; color: var(--fg-4); text-transform: uppercase; letter-spacing: .04em; }
  .mapping-row {
    display: grid; grid-template-columns: 1fr 1fr; align-items: center;
    padding: 6px 12px; border-bottom: 1px solid var(--border); gap: 10px;
  }
  .mapping-row:last-child { border-bottom: none; }
  .mapping-col-name { font-size: 12px; color: var(--fg); font-family: monospace; }
  .mapping-select {
    padding: 4px 8px; background: var(--surface-2); border: 1px solid var(--border);
    border-radius: var(--r); font-size: 11px; color: var(--fg);
  }
  .mapping-select:focus { border-color: var(--orange); outline: none; }

  .import-summary {
    display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px;
    background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--r);
    padding: 10px; margin-bottom: 10px; font-size: 11px; color: var(--fg-3);
  }
  .skip-count { color: var(--fg-4); }

  .danger-zone { border-color: rgba(239,68,68,.35); }
  .reset-row { display: flex; gap: 8px; align-items: center; }
  .btn-danger {
    padding: 7px 12px; border-radius: var(--r); border: 1px solid #ef4444;
    background: #fee2e2; color: #dc2626; font-size: 12px; font-weight: 700; cursor: pointer;
  }
  .btn-danger:disabled { opacity: .6; cursor: wait; }
</style>
