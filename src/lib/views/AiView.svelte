<script lang="ts">
  import { marked } from "marked";
  import {
    aiConversationId, aiMessages, aiLoading, addToast,
    projects, items, members,
  } from "../stores/app";
  import { chatWithAi, applyAiAction, listProjects, listItems, listMembers, listAiConversations } from "../api";
  import type { AiAction, AiConversation } from "../types";
  import { onMount, onDestroy } from "svelte";
  import GanymedeMark from "../components/GanymedeMark.svelte";

  marked.use({ gfm: true, breaks: true });

  let inputText = "";
  let chatContainer: HTMLDivElement;
  let conversations: AiConversation[] = [];
  let showHistory = false;

  // Streaming animation — character-by-character with natural rhythm
  let streamStore: Record<string, string> = {};
  let streamComplete = new Set<string>();

  async function streamIn(id: string, fullText: string) {
    streamStore = { ...streamStore, [id]: '' };
    let i = 0;
    while (i < fullText.length) {
      // Vary chunk size: punctuation pauses (2-4 chars), normal (5-10 chars)
      const ch = fullText[i];
      let chunkSize = 8;
      if ('.!?\n:'.includes(ch)) {
        chunkSize = 2;
        await new Promise<void>(r => setTimeout(r, 28));
      } else if (',;'.includes(ch)) {
        chunkSize = 3;
        await new Promise<void>(r => setTimeout(r, 14));
      } else {
        await new Promise<void>(r => requestAnimationFrame(() => r()));
      }
      i += chunkSize;
      streamStore = { ...streamStore, [id]: fullText.slice(0, Math.min(i, fullText.length)) };
      scrollToBottom();
    }
    streamComplete = new Set([...streamComplete, id]);
    streamStore = { ...streamStore, [id]: fullText };
    scrollToBottom();
  }

  function handlePrefill(e: Event) {
    inputText = (e as CustomEvent<string>).detail;
    sendMessage();
  }

  onMount(async () => {
    window.addEventListener("ganymede:prefill-ai", handlePrefill);
    try { conversations = await listAiConversations(); } catch {}
  });

  onDestroy(() => {
    window.removeEventListener("ganymede:prefill-ai", handlePrefill);
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
      <span class="ai-sidebar-title">History</span>
      <button class="btn-icon" on:click={newConversation} title="New conversation">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
      </button>
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
          <div class="ai-brand-row">
            <span class="ai-brand-dot"></span>
            <span class="ai-brand-name">Ganymede Assistant</span>
          </div>
          <span class="ai-brand-sub">Ops co-pilot · projects, issues, execution</span>
        </div>
      </div>
      <button class="btn-new-conv" on:click={newConversation}>
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        New chat
      </button>
    </div>

    <div class="ai-chat" bind:this={chatContainer}>
      {#if $aiMessages.length === 0}
        <div class="ai-welcome">
          <div class="ai-welcome-glyph" aria-hidden="true">
            <GanymedeMark size={36} />
          </div>
          <h2>Good to see you.</h2>
          <p>Ask me anything about your projects, issues, team load, or what to prioritise today. I can create work items directly from your description.</p>
          <div class="ai-quick-actions">
            <button class="ai-quick-btn" on:click={() => { inputText = "What needs my attention today?"; sendMessage(); }}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
              What needs attention?
            </button>
            <button class="ai-quick-btn" on:click={() => { inputText = "Summarise all open issues by priority"; sendMessage(); }}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86"/></svg>
              Summarise open issues
            </button>
            <button class="ai-quick-btn" on:click={() => { inputText = "What is everyone working on?"; sendMessage(); }}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87"/><path d="M16 3.13a4 4 0 010 7.75"/></svg>
              Team workload
            </button>
            <button class="ai-quick-btn" on:click={() => { inputText = "Which projects are at risk?"; sendMessage(); }}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
              Projects at risk
            </button>
            <button class="ai-quick-btn" on:click={() => { inputText = "Add a new P1 issue: "; }}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
              Create issue
            </button>
            <button class="ai-quick-btn" on:click={() => { inputText = "Add a follow-up: "; }}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 17H2a3 3 0 000 6h20a3 3 0 000-6z"/><path d="M5 17V5a2 2 0 012-2h10a2 2 0 012 2v12"/></svg>
              Track a follow-up
            </button>
          </div>
        </div>
      {:else}
        {#each $aiMessages as msg, idx (msg.id)}
          <div class="ai-msg" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
            {#if msg.role === 'assistant'}
              <div class="ai-msg-avatar" aria-hidden="true">
                <GanymedeMark size={14} />
              </div>
            {/if}
            <div class="ai-msg-body">
              {#if msg.role === 'user'}
                <div class="ai-user-bubble">{msg.content}</div>
              {:else}
                <div class="ai-msg-content">
                  {#if streamStore[msg.id] !== undefined && !streamComplete.has(msg.id)}
                    {@html renderMarkdown(streamStore[msg.id])}<span class="stream-cursor">▌</span>
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
                      <button class="ai-followup-btn" on:click={() => handleFollowup(q)}>{q}</button>
                    {/each}
                  </div>
                {/if}
              {/if}
            </div>
          </div>
        {/each}

        {#if $aiLoading}
          <div class="ai-msg assistant">
            <div class="ai-msg-avatar" aria-hidden="true">
              <GanymedeMark size={14} />
            </div>
            <div class="ai-msg-body">
              <div class="ai-thinking">
                <div class="thinking-header">
                  <span class="thinking-label">Thinking</span>
                  <span class="thinking-dots"><span></span><span></span><span></span></span>
                </div>
                <div class="thinking-shimmer"></div>
                <div class="thinking-shimmer short"></div>
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
          placeholder="Ask about your projects, issues, team, or what to focus on today…"
          class="ai-input"
          rows="2"
          on:keydown={handleKeydown}
        ></textarea>
        <div class="ai-input-controls">
          <span class="ai-input-hint">↵ send &nbsp;·&nbsp; ⇧↵ newline</span>
          <button class="ai-send" on:click={sendMessage} disabled={$aiLoading || !inputText.trim()} title="Send">
            {#if $aiLoading}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" class="spin"><path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/></svg>
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/></svg>
            {/if}
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
    background: var(--bg);
    position: relative;
  }
  .ai-view::before {
    content: '';
    position: absolute; inset: 0; pointer-events: none; z-index: 0;
    background:
      radial-gradient(ellipse 80% 40% at 70% -10%, rgba(249,115,22,.07) 0%, transparent 60%),
      radial-gradient(ellipse 60% 50% at -5% 30%, rgba(99,102,241,.06) 0%, transparent 55%),
      radial-gradient(ellipse 50% 60% at 100% 90%, rgba(139,92,246,.04) 0%, transparent 50%);
  }

  /* ── History sidebar ── */
  .ai-sidebar {
    width: 0; overflow: hidden;
    transition: width 220ms cubic-bezier(0.16,1,.3,1);
    background: var(--surface);
    border-inline-end: 1px solid var(--border);
    display: flex; flex-direction: column; flex-shrink: 0;
    z-index: 1;
  }
  .ai-sidebar.open { width: 230px; }
  .ai-sidebar-header {
    padding: 14px 14px 10px;
    display: flex; align-items: center; justify-content: space-between;
    border-bottom: 1px solid var(--border);
  }
  .ai-sidebar-title { font-size: 10px; font-weight: 700; color: var(--fg-3); text-transform: uppercase; letter-spacing: 0.08em; }
  .ai-conv-list { flex: 1; overflow-y: auto; padding: 6px; }
  .ai-conv-item {
    width: 100%; text-align: left; padding: 8px 10px; border-radius: var(--r);
    cursor: pointer; transition: background 120ms; background: none; border: none;
    display: flex; flex-direction: column; gap: 2px;
  }
  .ai-conv-item:hover { background: var(--surface-2); }
  .ai-conv-item.active { background: var(--orange-bg); }
  .ai-conv-title { font-size: 11.5px; font-weight: 500; color: var(--fg); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .ai-conv-item.active .ai-conv-title { color: var(--orange); }
  .ai-conv-date { font-size: 10px; color: var(--fg-4); }
  .ai-conv-empty { font-size: 11px; color: var(--fg-4); padding: 12px 10px; }

  /* ── Main chat — FULL WIDTH ── */
  .ai-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    z-index: 1;
  }

  .ai-topbar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 28px;
    border-bottom: 1px solid var(--border);
    background: color-mix(in oklab, var(--surface) 60%, transparent 40%);
    backdrop-filter: blur(12px);
    flex-shrink: 0;
  }
  .ai-topbar-left { display: flex; align-items: center; gap: 12px; }
  .ai-brand { display: flex; flex-direction: column; gap: 1px; }
  .ai-brand-row { display: flex; align-items: center; gap: 6px; }
  .ai-brand-dot {
    width: 6px; height: 6px; border-radius: 50%;
    background: #22c55e;
    box-shadow: 0 0 6px rgba(34,197,94,.6);
    animation: pulse-green 2.4s ease-in-out infinite;
  }
  @keyframes pulse-green {
    0%, 100% { box-shadow: 0 0 4px rgba(34,197,94,.5); }
    50%       { box-shadow: 0 0 10px rgba(34,197,94,.9); }
  }
  .ai-brand-name { font-size: 13px; font-weight: 700; color: var(--fg); letter-spacing: -0.02em; }
  .ai-brand-sub { font-size: 10px; color: var(--fg-4); padding-left: 12px; }
  .btn-new-conv {
    display: flex; align-items: center; gap: 6px;
    padding: 6px 14px; border-radius: var(--r);
    background: none; color: var(--fg-3);
    font-size: 11px; font-weight: 600;
    cursor: pointer; border: 1px solid var(--border); transition: all 140ms;
  }
  .btn-new-conv:hover { background: var(--surface-2); color: var(--fg); border-color: var(--border-2); }

  /* ── Chat messages ── */
  .ai-chat {
    flex: 1;
    overflow-y: auto;
    padding: 32px 10% 24px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  /* ── Welcome ── */
  .ai-welcome {
    margin: auto;
    text-align: center;
    max-width: 680px;
    padding: 56px 32px;
    animation: welcome-in 0.4s ease;
  }
  @keyframes welcome-in {
    from { opacity: 0; transform: translateY(16px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  .ai-welcome-glyph {
    width: 68px; height: 68px; border-radius: 20px;
    background: linear-gradient(135deg, rgba(249,115,22,.12), rgba(99,102,241,.08));
    border: 1px solid rgba(249,115,22,.2);
    display: flex; align-items: center; justify-content: center;
    margin: 0 auto 22px;
    color: var(--orange);
    box-shadow: 0 12px 32px rgba(249,115,22,.12);
  }
  .ai-welcome h2 {
    font-size: 28px; font-weight: 780; margin: 0 0 12px; color: var(--fg);
    letter-spacing: -0.04em;
  }
  .ai-welcome p { font-size: 13.5px; color: var(--fg-3); margin: 0; line-height: 1.65; max-width: 480px; margin: 0 auto; }
  .ai-quick-actions {
    display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 9px;
    margin-top: 28px;
  }
  .ai-quick-btn {
    display: flex; align-items: center; gap: 9px;
    padding: 12px 14px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-lg); font-size: 12px; font-weight: 500; color: var(--fg-2);
    cursor: pointer; transition: all 140ms; text-align: left;
  }
  .ai-quick-btn:hover { border-color: var(--orange); color: var(--fg); background: var(--orange-bg); box-shadow: 0 4px 14px rgba(249,115,22,.1); }
  .ai-quick-btn svg { flex-shrink: 0; color: var(--fg-4); transition: color 140ms; }
  .ai-quick-btn:hover svg { color: var(--orange); }

  /* ── Messages ── */
  .ai-msg {
    display: flex; gap: 12px;
    animation: msg-in 0.25s cubic-bezier(0.16,1,.3,1);
  }
  @keyframes msg-in {
    from { opacity: 0; transform: translateY(10px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  .ai-msg.user { justify-content: flex-end; }

  .ai-msg-avatar {
    width: 30px; height: 30px; border-radius: 10px;
    background: linear-gradient(135deg, rgba(249,115,22,.15), rgba(99,102,241,.1));
    border: 1px solid rgba(249,115,22,.2);
    display: flex; align-items: center; justify-content: center;
    flex-shrink: 0; margin-top: 1px;
    color: var(--orange);
  }
  .ai-msg-body { max-width: min(78%, 900px); }
  .ai-user-bubble {
    background: linear-gradient(135deg, #f97316 0%, #ea580c 100%);
    color: #fff; padding: 11px 16px;
    border-radius: 18px 18px 4px 18px;
    font-size: 13.5px; line-height: 1.55;
    box-shadow: 0 6px 20px rgba(234,88,12,.2);
    white-space: pre-wrap;
  }
  .ai-msg.assistant .ai-msg-content {
    background: var(--surface);
    padding: 14px 18px;
    border-radius: 4px 18px 18px 18px;
    border: 1px solid var(--border);
    box-shadow: 0 2px 12px rgba(0,0,0,.06);
    font-size: 13.5px; line-height: 1.65; color: var(--fg);
  }
  .ai-msg.assistant .ai-msg-content :global(p) { margin: 0 0 10px; }
  .ai-msg.assistant .ai-msg-content :global(p:last-child) { margin-bottom: 0; }
  .ai-msg.assistant .ai-msg-content :global(h2) { font-size: 14px; font-weight: 700; margin: 12px 0 6px; color: var(--fg); }
  .ai-msg.assistant .ai-msg-content :global(h3) { font-size: 13px; font-weight: 700; margin: 10px 0 4px; color: var(--fg-2); }
  .ai-msg.assistant .ai-msg-content :global(strong) { font-weight: 700; color: var(--fg); }
  .ai-msg.assistant .ai-msg-content :global(em) { color: var(--fg-2); }
  .ai-msg.assistant .ai-msg-content :global(code) {
    background: var(--surface-2); padding: 2px 6px; border-radius: 4px;
    font-size: 12px; color: var(--orange);
  }
  .ai-msg.assistant .ai-msg-content :global(ul), .ai-msg.assistant .ai-msg-content :global(ol) { margin: 8px 0; padding-left: 20px; }
  .ai-msg.assistant .ai-msg-content :global(li) { margin: 4px 0; line-height: 1.55; }
  .ai-msg.assistant .ai-msg-content :global(blockquote) {
    border-left: 3px solid var(--orange); padding-left: 12px;
    margin: 10px 0; color: var(--fg-2); font-style: italic;
  }
  .ai-msg.assistant .ai-msg-content :global(hr) { border: none; border-top: 1px solid var(--border); margin: 12px 0; }

  /* ── Tables in AI responses ── */
  .ai-msg.assistant .ai-msg-content :global(table) {
    width: 100%; border-collapse: collapse; margin: 12px 0;
    font-size: 12.5px; border-radius: 8px; overflow: hidden;
    border: 1px solid var(--border);
  }
  .ai-msg.assistant .ai-msg-content :global(thead) {
    background: var(--surface-2);
  }
  .ai-msg.assistant .ai-msg-content :global(th) {
    padding: 8px 12px; text-align: left; font-weight: 700;
    color: var(--fg); font-size: 11px; text-transform: uppercase;
    letter-spacing: 0.04em; border-bottom: 1px solid var(--border);
  }
  .ai-msg.assistant .ai-msg-content :global(td) {
    padding: 8px 12px; color: var(--fg-2); border-bottom: 1px solid var(--border);
    vertical-align: top; line-height: 1.5;
  }
  .ai-msg.assistant .ai-msg-content :global(tr:last-child td) {
    border-bottom: none;
  }
  .ai-msg.assistant .ai-msg-content :global(tr:nth-child(even)) {
    background: rgba(0,0,0,.02);
  }
  .ai-msg.assistant .ai-msg-content :global(tr:hover td) {
    background: var(--orange-bg);
  }

  /* ── Applied summary ── */
  .ai-applied-list {
    display: flex; flex-direction: column; gap: 4px; margin-top: 10px;
    padding: 10px 14px;
    background: rgba(34,197,94,.05);
    border: 1px solid rgba(34,197,94,.15);
    border-radius: 10px;
  }
  .ai-applied-item {
    display: flex; align-items: center; gap: 8px;
    font-size: 11.5px; color: var(--green); font-weight: 500;
  }

  /* ── Follow-ups ── */
  .ai-followups { display: flex; flex-wrap: wrap; gap: 7px; margin-top: 12px; }
  .ai-followup-btn {
    padding: 7px 13px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 20px; font-size: 12px; color: var(--fg-3); cursor: pointer;
    transition: all 140ms; text-align: left;
  }
  .ai-followup-btn:hover { border-color: var(--orange); color: var(--orange); background: var(--orange-bg); }

  /* ── Thinking indicator ── */
  .ai-thinking {
    padding: 14px 18px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 4px 18px 18px 18px;
    display: flex; flex-direction: column; gap: 10px;
    min-width: 180px;
  }
  .thinking-header {
    display: flex; align-items: center; gap: 8px;
  }
  .thinking-label {
    font-size: 12px; font-weight: 600; color: var(--fg-3);
    letter-spacing: 0.01em;
  }
  .thinking-dots {
    display: flex; gap: 4px;
  }
  .thinking-dots span {
    width: 5px; height: 5px; border-radius: 50%;
    background: var(--orange);
    animation: thinking-bounce 1.2s ease-in-out infinite;
    opacity: 0.7;
  }
  .thinking-dots span:nth-child(2) { animation-delay: 0.15s; }
  .thinking-dots span:nth-child(3) { animation-delay: 0.30s; }
  @keyframes thinking-bounce {
    0%, 80%, 100% { transform: scale(0.7); opacity: 0.4; }
    40%           { transform: scale(1);   opacity: 1; }
  }
  .thinking-shimmer {
    height: 10px; border-radius: 6px;
    background: linear-gradient(90deg, var(--surface-2) 25%, var(--border) 50%, var(--surface-2) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.6s infinite;
  }
  .thinking-shimmer.short { width: 60%; height: 10px; }
  @keyframes shimmer {
    0%   { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  /* ── Input ── */
  .ai-input-area {
    padding: 14px 10% 20px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }
  .ai-input-wrapper {
    background: var(--surface);
    border: 1.5px solid var(--border);
    border-radius: 16px;
    padding: 13px 14px 10px;
    display: flex; flex-direction: column; gap: 8px;
    transition: border-color 160ms, box-shadow 160ms;
    box-shadow: 0 2px 12px rgba(0,0,0,.06);
  }
  .ai-input-wrapper:focus-within {
    border-color: rgba(249,115,22,.5);
    box-shadow: 0 0 0 3px rgba(249,115,22,.08), 0 2px 12px rgba(0,0,0,.06);
  }
  .ai-input {
    width: 100%; resize: none; background: transparent;
    border: none; outline: none;
    padding: 0; font-size: 13.5px; color: var(--fg);
    font-family: inherit; line-height: 1.55; box-sizing: border-box;
  }
  .ai-input::placeholder { color: var(--fg-4); }
  .ai-input-controls {
    display: flex; align-items: center; justify-content: space-between;
  }
  .ai-input-hint { font-size: 10px; color: var(--fg-4); }
  .ai-send {
    width: 34px; height: 34px; border-radius: 11px;
    background: linear-gradient(135deg, #fb923c 0%, #ea580c 100%); color: #fff;
    border: none; cursor: pointer;
    transition: opacity 120ms, transform 80ms;
    display: flex; align-items: center; justify-content: center;
    flex-shrink: 0;
    box-shadow: 0 4px 12px rgba(234,88,12,.25);
  }
  .ai-send:not(:disabled):hover { opacity: 0.9; transform: translateY(-1px); }
  .ai-send:not(:disabled):active { transform: translateY(0); }
  .ai-send:disabled { opacity: 0.3; cursor: not-allowed; box-shadow: none; }

  /* Streaming cursor */
  .stream-cursor {
    display: inline-block; color: var(--orange);
    animation: blink-cursor 0.65s steps(2, start) infinite;
    font-size: 14px;
  }
  @keyframes blink-cursor { 50% { opacity: 0; } }

  /* Spinning loader in send button */
  .spin { animation: spin-anim 1s linear infinite; }
  @keyframes spin-anim { to { transform: rotate(360deg); } }

  @media (max-width: 900px) {
    .ai-sidebar.open { width: 200px; }
    .ai-chat, .ai-input-area { padding-inline: 5%; }
    .ai-quick-actions { grid-template-columns: 1fr 1fr; }
    .ai-brand-sub { display: none; }
  }
  @media (max-width: 600px) {
    .ai-quick-actions { grid-template-columns: 1fr; }
  }
</style>
