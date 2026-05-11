<script lang="ts">
  import { marked } from "marked";
  import {
    aiConversationId, aiMessages, aiLoading, addToast,
    projects, items, members,
  } from "../stores/app";
  import { chatWithAi, applyAiAction, listProjects, listItems, listMembers, listAiConversations } from "../api";
  import type { AiAction, AiConversation } from "../types";
  import { onMount } from "svelte";

  marked.use({ gfm: true, breaks: true });

  let inputText = "";
  let chatContainer: HTMLDivElement;
  let conversations: AiConversation[] = [];
  let showHistory = false;

  // Streaming animation
  let streamStore: Record<string, string> = {};
  let streamComplete = new Set<string>();

  async function streamIn(id: string, fullText: string) {
    streamStore = { ...streamStore, [id]: '' };
    const CHUNK = 6;
    for (let i = CHUNK; i <= fullText.length + CHUNK; i += CHUNK) {
      streamStore = { ...streamStore, [id]: fullText.slice(0, Math.min(i, fullText.length)) };
      scrollToBottom();
      await new Promise<void>(r => requestAnimationFrame(() => r()));
    }
    streamComplete = new Set([...streamComplete, id]);
    streamStore = { ...streamStore, [id]: fullText };
    scrollToBottom();
  }

  onMount(async () => {
    try {
      conversations = await listAiConversations();
    } catch {}
  });

  function scrollToBottom() {
    setTimeout(() => {
      if (chatContainer) chatContainer.scrollTop = chatContainer.scrollHeight;
    }, 50);
  }

  async function sendMessage() {
    const msg = inputText.trim();
    if (!msg || $aiLoading) return;
    inputText = "";

    aiMessages.update((msgs) => [
      ...msgs,
      {
        id: `temp-${Date.now()}`,
        conversation_id: $aiConversationId || "",
        role: "user" as const,
        content: msg,
        actions: [],
        followups: [],
        created_at: new Date().toISOString(),
      },
    ]);
    scrollToBottom();

    aiLoading.set(true);
    try {
      const response = await chatWithAi({
        message: msg,
        conversation_id: $aiConversationId || undefined,
      });
      aiConversationId.set(response.conversation_id);

      // Auto-apply all actions immediately — no manual "Apply" needed
      const applied: string[] = [];
      for (const action of response.actions) {
        try {
          const res = await applyAiAction(action);
          applied.push(res);
        } catch {
          // silently skip failed actions; errors were already scoped away
        }
      }

      const msgId = `asst-${Date.now()}`;
    aiMessages.update((msgs) => [
        ...msgs,
        {
          id: msgId,
          conversation_id: response.conversation_id,
          role: "assistant" as const,
          content: response.message,
          actions: response.actions.map((a) => ({ ...a, status: "applied" as const })),
          applied_summary: applied,
          followups: response.followups,
          created_at: new Date().toISOString(),
        },
      ]);
      streamIn(msgId, response.message);

      // Refresh stores
      if (applied.length > 0) {
        listProjects().then((p) => projects.set(p)).catch(() => {});
        listItems().then((i) => items.set(i)).catch(() => {});
        listMembers().then((m) => members.set(m)).catch(() => {});
      }

      // Refresh conversation list
      conversations = await listAiConversations().catch(() => conversations);
    } catch (e: any) {
      aiMessages.update((msgs) => [
        ...msgs,
        {
          id: `err-${Date.now()}`,
          conversation_id: $aiConversationId || "",
          role: "assistant" as const,
          content: `**Error:** ${e}`,
          actions: [],
          followups: [],
          created_at: new Date().toISOString(),
        },
      ]);
      scrollToBottom();
    }
    aiLoading.set(false);
  }

  async function handleApplyAction(action: AiAction, msgIdx: number) {
    try {
      const result = await applyAiAction(action);
      addToast(result, "✓");
      aiMessages.update((msgs) => {
        const updated = [...msgs];
        const msg = updated[msgIdx];
        if (msg) {
          msg.actions = msg.actions.map((a) =>
            a.id === action.id ? { ...a, status: "applied" as const } : a
          );
        }
        return updated;
      });
      listProjects().then((p) => projects.set(p)).catch(() => {});
      listItems().then((i) => items.set(i)).catch(() => {});
      listMembers().then((m) => members.set(m)).catch(() => {});
    } catch (e: any) {
      addToast(`Failed: ${e}`, "✗");
    }
  }

  function handleFollowup(q: string) {
    inputText = q;
    sendMessage();
  }

  function newConversation() {
    aiConversationId.set(null);
    aiMessages.set([]);
  }

  async function loadConversation(id: string) {
    // Switch to this conversation — get messages
    try {
      const { getAiMessages } = await import("../api");
      const msgs = await getAiMessages(id);
      aiConversationId.set(id);
      aiMessages.set(msgs);
      showHistory = false;
      scrollToBottom();
    } catch {}
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  function renderMarkdown(text: string): string {
    return marked.parse(text) as string;
  }

  function formatTime(iso: string) {
    const d = new Date(iso);
    return d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
  }
</script>

<div class="ai-view">
  <!-- Sidebar: conversation history -->
  <div class="ai-sidebar" class:open={showHistory}>
    <div class="ai-sidebar-header">
      <span class="ai-sidebar-title">Conversations</span>
      <button class="btn-icon" on:click={newConversation} title="New conversation">+</button>
    </div>
    <div class="ai-conv-list">
      {#each conversations as conv}
        <button
          class="ai-conv-item"
          class:active={$aiConversationId === conv.id}
          on:click={() => loadConversation(conv.id)}
        >
          <div class="ai-conv-title">{conv.title || "Untitled"}</div>
          <div class="ai-conv-date">{formatTime(conv.updated_at)}</div>
        </button>
      {:else}
        <div class="ai-conv-empty">No conversations yet</div>
      {/each}
    </div>
  </div>

  <!-- Main chat area -->
  <div class="ai-main">
    <div class="ai-topbar">
      <div class="ai-topbar-left">
        <button class="btn-icon" on:click={() => showHistory = !showHistory} title="History">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg>
        </button>
        <div class="ai-brand">
          <span class="ai-brand-name">Ganymede Assistant</span>
          <span class="ai-brand-sub">Ops co-pilot for projects, issues, and execution</span>
        </div>
      </div>
      <button class="btn-new-conv" on:click={newConversation}>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        New
      </button>
    </div>

    <div class="ai-chat" bind:this={chatContainer}>
      {#if $aiMessages.length === 0}
        <div class="ai-welcome">
          <div class="ai-welcome-icon" aria-hidden="true">
            <img src="/billdesk-logo.png" alt="" />
          </div>
          <h2>Plan Faster. Track Better.</h2>
          <p>Use natural language to create structured work: projects, issues, follow-ups, ideas, and decisions.</p>
          <div class="ai-quick-actions">
            <button class="ai-quick-btn" on:click={() => { inputText = "List all open issues"; sendMessage(); }}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86"/></svg>
              List open issues
            </button>
            <button class="ai-quick-btn" on:click={() => { inputText = "Add a new P1 issue: "; }}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
              Create issue
            </button>
            <button class="ai-quick-btn" on:click={() => { inputText = "What is everyone working on?"; sendMessage(); }}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87"/><path d="M16 3.13a4 4 0 010 7.75"/></svg>
              Team overview
            </button>
            <button class="ai-quick-btn" on:click={() => { inputText = "Add an idea: "; }}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2a7 7 0 017 7c0 2.38-1.19 4.47-3 5.74V17a2 2 0 01-2 2h-4a2 2 0 01-2-2v-2.26C6.19 13.47 5 11.38 5 9a7 7 0 017-7z"/></svg>
              Brainstorm idea
            </button>
          </div>
        </div>
      {:else}
        {#each $aiMessages as msg, idx (msg.id)}
          <div class="ai-msg" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
            {#if msg.role === 'assistant'}
              <div class="ai-msg-avatar"><img src="/billdesk-logo.png" alt="" /></div>
            {/if}
            <div class="ai-msg-body">
              <div class="ai-msg-content">
                {#if streamStore[msg.id] !== undefined && !streamComplete.has(msg.id)}
                  {streamStore[msg.id]}<span class="stream-cursor">▌</span>
                {:else}
                  {@html renderMarkdown(msg.content)}
                {/if}
              </div>

              {#if msg.applied_summary && msg.applied_summary.length > 0 && (streamComplete.has(msg.id) || streamStore[msg.id] === undefined)}
                <div class="ai-applied-list">
                  {#each msg.applied_summary as item}
                    <div class="ai-applied-item">
                      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
                      {item}
                    </div>
                  {/each}
                </div>
              {/if}

              {#if msg.followups && msg.followups.length > 0 && (streamComplete.has(msg.id) || streamStore[msg.id] === undefined)}
                <div class="ai-followups">
                  {#each msg.followups as q}
                    <button class="ai-followup-btn" on:click={() => handleFollowup(q)}>
                      {q}
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        {/each}

        {#if $aiLoading}
          <div class="ai-msg assistant">
            <div class="ai-msg-avatar">AI</div>
            <div class="ai-msg-body">
              <div class="ai-thinking">
                <span class="dot"></span><span class="dot"></span><span class="dot"></span>
              </div>
            </div>
          </div>
        {/if}
      {/if}
    </div>

    <div class="ai-input-area">
      <div class="ai-input-wrapper">
        <textarea
          bind:value={inputText}
          placeholder="Ask about your projects, issues, team, or what to do next…"
          class="ai-input"
          rows="3"
          on:keydown={handleKeydown}
        ></textarea>
        <div class="ai-input-controls">
          <span class="ai-input-hint">↵ send &nbsp;·&nbsp; ⇧↵ newline</span>
          <button class="ai-send" on:click={sendMessage} disabled={$aiLoading || !inputText.trim()}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/></svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .ai-view {
    display: flex;
    height: 100%;
    background:
      radial-gradient(1200px 600px at 85% -20%, rgba(249,115,22,.08), transparent 60%),
      radial-gradient(900px 500px at -10% 20%, rgba(59,130,246,.08), transparent 60%),
      var(--bg);
  }

  /* ── History sidebar ── */
  .ai-sidebar {
    width: 0;
    overflow: hidden;
    transition: width 200ms ease;
    background: color-mix(in oklab, var(--surface) 92%, #0f172a 8%);
    border-inline-end: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }
  .ai-sidebar.open { width: 240px; }

  .ai-sidebar-header {
    padding: 12px 14px;
    display: flex; align-items: center; justify-content: space-between;
    border-bottom: 1px solid var(--border);
    font-size: 11px; font-weight: 700; color: var(--fg-2);
  }
  .ai-sidebar-title { text-transform: uppercase; letter-spacing: 0.06em; font-size: 10px; }
  .ai-conv-list { flex: 1; overflow-y: auto; padding: 6px; }
  .ai-conv-item {
    width: 100%; text-align: left; padding: 8px 10px; border-radius: var(--r);
    cursor: pointer; transition: background 120ms; background: none; border: none;
    display: flex; flex-direction: column; gap: 2px;
  }
  .ai-conv-item:hover { background: var(--surface-2); }
  .ai-conv-item.active { background: var(--orange-bg); }
  .ai-conv-title { font-size: 11px; font-weight: 500; color: var(--fg); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .ai-conv-item.active .ai-conv-title { color: var(--orange); }
  .ai-conv-date { font-size: 10px; color: var(--fg-4); }
  .ai-conv-empty { font-size: 11px; color: var(--fg-4); padding: 12px 10px; }

  /* ── Main chat ── */
  .ai-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    max-width: 900px;
    margin: 0 auto;
    width: 100%;
    min-width: 0;
  }

  .ai-topbar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 22px; border-bottom: 1px solid var(--border);
    backdrop-filter: blur(8px);
    flex-shrink: 0;
  }
  .ai-topbar-left { display: flex; align-items: center; gap: 10px; }
  .ai-brand { display: flex; flex-direction: column; align-items: flex-start; gap: 1px; }

  .ai-brand-name { font-size: 13px; font-weight: 700; color: var(--fg); letter-spacing: -0.01em; }
  .ai-brand-sub { font-size: 10px; color: var(--fg-4); }

  .btn-new-conv {
    display: flex; align-items: center; gap: 5px;
    padding: 5px 12px; border-radius: var(--r);
    background: linear-gradient(135deg, #fb923c 0%, #f97316 100%); color: #fff;
    font-size: 11px; font-weight: 600;
    cursor: pointer; border: none; transition: opacity 120ms;
  }
  .btn-new-conv:hover { opacity: 0.88; }

  /* ── Chat messages ── */
  .ai-chat {
    flex: 1;
    overflow-y: auto;
    padding: 28px 22px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  /* ── Welcome ── */
  .ai-welcome {
    margin: auto;
    text-align: center;
    max-width: 620px;
    padding: 48px 24px;
  }
  .ai-welcome-icon {
    width: 64px; height: 64px; border-radius: 18px;
    background: white;
    border: 1px solid rgba(249,115,22,.2);
    display: flex; align-items: center; justify-content: center;
    margin: 0 auto 20px;
    box-shadow: 0 10px 30px rgba(0,0,0,.18);
    padding: 10px;
  }
  .ai-welcome-icon img { width: 100%; height: 100%; object-fit: contain; }
  .ai-welcome h2 { font-size: 26px; font-weight: 780; margin: 0 0 10px; color: var(--fg); letter-spacing: -0.03em; }
  .ai-welcome p { font-size: 13px; color: var(--fg-3); margin: 0; line-height: 1.6; }

  .ai-quick-actions {
    display: grid; grid-template-columns: 1fr 1fr; gap: 10px;
    margin-top: 24px; max-width: 560px; margin-inline: auto;
  }
  .ai-quick-btn {
    display: flex; align-items: center; gap: 8px;
    padding: 11px 14px; background: color-mix(in oklab, var(--surface) 88%, #111827 12%);
    border: 1px solid color-mix(in oklab, var(--border) 80%, #111827 20%);
    border-radius: var(--r-lg); font-size: 12px; font-weight: 500; color: var(--fg-2);
    cursor: pointer; transition: all 120ms; text-align: left;
  }
  .ai-quick-btn:hover { border-color: var(--orange); color: var(--orange); background: var(--orange-bg); }
  .ai-quick-btn svg { flex-shrink: 0; color: var(--fg-3); }
  .ai-quick-btn:hover svg { color: var(--orange); }

  /* ── Messages ── */
  .ai-msg { display: flex; gap: 10px; }
  .ai-msg.user { justify-content: flex-end; }
  .ai-msg-avatar {
    width: 28px; height: 28px; border-radius: 8px;
    background: white;
    display: flex; align-items: center; justify-content: center;
    flex-shrink: 0; margin-top: 2px;
    border: 1px solid rgba(249,115,22,.25);
    padding: 3px;
    box-shadow: 0 2px 8px rgba(0,0,0,.12);
  }
  .ai-msg-avatar img { width: 100%; height: 100%; object-fit: contain; }
  .ai-msg-body { max-width: min(82%, 760px); }
  .ai-msg.user .ai-msg-body {
    background: linear-gradient(135deg, #fb923c 0%, #f97316 100%);
    color: #fff; padding: 10px 14px;
    border-radius: 14px 14px 3px 14px; font-size: 13px; line-height: 1.55;
    box-shadow: 0 8px 18px rgba(249,115,22,.18);
  }
  .ai-msg.assistant .ai-msg-content {
    background: color-mix(in oklab, var(--surface) 90%, #111827 10%);
    padding: 12px 16px;
    border-radius: 3px 14px 14px 14px;
    border: 1px solid color-mix(in oklab, var(--border) 85%, #111827 15%);
    box-shadow: 0 8px 20px rgba(0,0,0,.14);
    font-size: 13px; line-height: 1.6; color: var(--fg);
  }
  .ai-msg.assistant .ai-msg-content :global(strong) { font-weight: 700; }
  .ai-msg.assistant .ai-msg-content :global(code) {
    background: var(--surface-2); padding: 1px 5px; border-radius: 4px; font-size: 12px;
  }
  .ai-msg.assistant .ai-msg-content :global(ul) { margin: 8px 0; padding-left: 18px; }
  .ai-msg.assistant .ai-msg-content :global(li) { margin: 3px 0; }

  /* ── Applied summary ── */
  .ai-applied-list { display: flex; flex-direction: column; gap: 3px; margin-top: 8px; }
  .ai-applied-item {
    display: flex; align-items: center; gap: 6px;
    font-size: 11px; color: var(--green); font-weight: 500;
  }
  .ai-applied-item svg { flex-shrink: 0; color: var(--green); }

  /* ── Follow-ups ── */
  .ai-followups { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 10px; }
  .ai-followup-btn {
    padding: 6px 12px; background: none; border: 1px dashed var(--border-2);
    border-radius: 20px; font-size: 12px; color: var(--fg-3); cursor: pointer;
    transition: all 120ms;
  }
  .ai-followup-btn:hover { border-color: var(--orange); color: var(--orange); background: var(--orange-bg); }

  /* ── Thinking ── */
  .ai-thinking { display: flex; gap: 5px; padding: 14px 0; }
  .dot {
    width: 7px; height: 7px; border-radius: 50%; background: var(--fg-4);
    animation: blink 1.4s infinite both;
  }
  .dot:nth-child(2) { animation-delay: 0.2s; }
  .dot:nth-child(3) { animation-delay: 0.4s; }
  @keyframes blink {
    0%, 80%, 100% { opacity: 0.25; }
    40% { opacity: 1; }
  }

  /* ── Input ── */
  .ai-input-area {
    padding: 14px 22px 20px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
    background: color-mix(in oklab, var(--surface) 88%, #111827 12%);
  }
  .ai-input-wrapper {
    background: color-mix(in oklab, var(--surface) 92%, #0b1220 8%);
    border: 1.5px solid color-mix(in oklab, var(--border) 80%, #0f172a 20%);
    border-radius: 14px;
    padding: 12px 14px 10px;
    display: flex; flex-direction: column; gap: 8px;
    transition: border-color 150ms, box-shadow 150ms;
  }
  .ai-input-wrapper:focus-within {
    border-color: var(--orange);
    box-shadow: 0 0 0 3px rgba(249,115,22,.1);
  }
  .ai-input {
    width: 100%; resize: none; background: transparent;
    border: none; outline: none;
    padding: 0; font-size: 13px; color: var(--fg);
    font-family: inherit; line-height: 1.55; box-sizing: border-box;
  }
  .ai-input::placeholder { color: var(--fg-4); }
  .ai-input-controls {
    display: flex; align-items: center; justify-content: space-between;
  }
  .ai-input-hint { font-size: 10px; color: var(--fg-4); letter-spacing: 0.01em; }
  .ai-send {
    width: 32px; height: 32px; border-radius: 10px;
    background: linear-gradient(135deg, #fb923c 0%, #f97316 100%); color: #fff;
    border: none; cursor: pointer;
    transition: opacity 120ms; display: flex; align-items: center; justify-content: center;
    flex-shrink: 0;
  }
  .ai-send:disabled { opacity: 0.35; cursor: not-allowed; }
  .ai-send:not(:disabled):hover { opacity: 0.88; }

  /* Streaming cursor */
  .stream-cursor {
    display: inline-block; color: var(--orange);
    animation: blink-cursor 0.7s steps(2, start) infinite;
  }
  @keyframes blink-cursor { 50% { opacity: 0; } }

  /* Message entry animation */
  @keyframes msg-in {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .ai-msg { animation: msg-in 0.22s ease; }

  @media (max-width: 980px) {
    .ai-sidebar.open { width: 190px; }
    .ai-main { max-width: 100%; }
    .ai-quick-actions { grid-template-columns: 1fr; max-width: 360px; }
    .ai-brand-sub { display: none; }
  }
</style>
