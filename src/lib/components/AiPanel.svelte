<script lang="ts">
  import {
    aiPanelOpen, aiConversationId, aiMessages, aiLoading, addToast,
    projects, items, members,
  } from "../stores/app";
  import { chatWithAi, applyAiAction, listProjects, listItems, listMembers } from "../api";
  import type { AiAction } from "../types";

  let inputText = "";
  let chatContainer: HTMLDivElement;

  function scrollToBottom() {
    setTimeout(() => {
      if (chatContainer) chatContainer.scrollTop = chatContainer.scrollHeight;
    }, 50);
  }

  async function sendMessage() {
    const msg = inputText.trim();
    if (!msg || $aiLoading) return;
    inputText = "";

    // Optimistically add user message
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

      // Add assistant message
      aiMessages.update((msgs) => [
        ...msgs,
        {
          id: `asst-${Date.now()}`,
          conversation_id: response.conversation_id,
          role: "assistant" as const,
          content: response.message,
          actions: response.actions,
          followups: response.followups,
          created_at: new Date().toISOString(),
        },
      ]);
      scrollToBottom();
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
      // Mark action as applied
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
      // Refresh data stores
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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  function renderMarkdown(text: string): string {
    return text
      .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
      .replace(/\*(.*?)\*/g, '<em>$1</em>')
      .replace(/`(.*?)`/g, '<code>$1</code>')
      .replace(/^- (.*)/gm, '<li>$1</li>')
      .replace(/(<li>.*<\/li>)/gs, '<ul>$1</ul>')
      .replace(/\n/g, '<br>');
  }
</script>

{#if $aiPanelOpen}
  <div class="ai-panel">
    <div class="ai-header">
      <div class="ai-title">
        <span class="ai-logo">◉</span>
        Ganymede AI
      </div>
      <div class="ai-header-actions">
        <button class="btn-icon" on:click={newConversation} title="New conversation">+</button>
        <button class="btn-icon" on:click={() => aiPanelOpen.set(false)} title="Close">✕</button>
      </div>
    </div>

    <div class="ai-chat" bind:this={chatContainer}>
      {#if $aiMessages.length === 0}
        <div class="ai-welcome">
          <div class="ai-welcome-logo">◉</div>
          <h3>Mission Control AI</h3>
          <p>Tell me what's going on — I'll organize it into projects, issues, and tasks.</p>
          <div class="ai-suggestions">
            <button class="ai-suggest" on:click={() => { inputText = "Kafka production outage — consumer lag spiking on payment-service"; sendMessage(); }}>
              Kafka production outage
            </button>
            <button class="ai-suggest" on:click={() => { inputText = "Need to start the SiHub token renewal project — tokens expiring next month"; sendMessage(); }}>
              SiHub token renewal project
            </button>
            <button class="ai-suggest" on:click={() => { inputText = "Consul load balancing algorithm needs to change from round-robin to least-connections"; sendMessage(); }}>
              Consul LB algorithm change
            </button>
          </div>
        </div>
      {:else}
        {#each $aiMessages as msg, idx}
          <div class="ai-msg" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
            {#if msg.role === 'assistant'}
              <div class="ai-msg-avatar">◉</div>
            {/if}
            <div class="ai-msg-body">
              <div class="ai-msg-content">{@html renderMarkdown(msg.content)}</div>

              {#if msg.actions.length > 0}
                <div class="ai-actions">
                  {#each msg.actions as action}
                    <div class="ai-action-card" class:applied={action.status === 'applied'}>
                      <div class="ai-action-type">{action.type.replace('_', ' ')}</div>
                      <div class="ai-action-label">{action.label}</div>
                      {#if action.status === 'applied'}
                        <span class="ai-action-done">✓ Applied</span>
                      {:else}
                        <button class="btn btn-primary btn-sm" on:click={() => handleApplyAction(action, idx)}>
                          Apply
                        </button>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}

              {#if msg.followups.length > 0}
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
            <div class="ai-msg-avatar">◉</div>
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
      <textarea
        bind:value={inputText}
        placeholder="Tell me what needs to be tracked…"
        class="ai-input"
        rows="2"
        on:keydown={handleKeydown}
      ></textarea>
      <button class="ai-send" on:click={sendMessage} disabled={$aiLoading || !inputText.trim()}>
        ↑
      </button>
    </div>
  </div>
{/if}

<style>
  .ai-panel {
    width: 380px;
    min-width: 320px;
    border-inline-start: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .ai-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 14px; border-bottom: 1px solid var(--border);
  }
  .ai-title { display: flex; align-items: center; gap: 8px; font-size: 13px; font-weight: 700; }
  .ai-logo { font-size: 16px; color: var(--orange); }
  .ai-header-actions { display: flex; gap: 4px; }

  .ai-chat { flex: 1; overflow-y: auto; padding: 14px; display: flex; flex-direction: column; gap: 12px; }

  .ai-welcome { text-align: center; padding: 30px 10px; }
  .ai-welcome-logo { font-size: 32px; color: var(--orange); margin-bottom: 8px; }
  .ai-welcome h3 { font-size: 15px; font-weight: 750; margin: 0 0 6px; }
  .ai-welcome p { font-size: 12px; color: var(--fg-3); margin: 0 0 16px; }
  .ai-suggestions { display: flex; flex-direction: column; gap: 6px; }
  .ai-suggest {
    padding: 8px 12px; background: var(--surface-2); border: 1px solid var(--border);
    border-radius: var(--r); font-size: 11px; color: var(--fg-2); cursor: pointer;
    text-align: left; transition: all 120ms;
  }
  .ai-suggest:hover { border-color: var(--orange); color: var(--orange); background: var(--orange-bg); }

  .ai-msg { display: flex; gap: 8px; max-width: 100%; }
  .ai-msg.user { justify-content: flex-end; }
  .ai-msg-avatar {
    width: 24px; height: 24px; border-radius: 50%; background: var(--orange-bg); color: var(--orange);
    display: flex; align-items: center; justify-content: center; font-size: 12px; flex-shrink: 0;
  }
  .ai-msg-body { max-width: 90%; }
  .ai-msg.user .ai-msg-body {
    background: var(--orange); color: #fff; padding: 8px 12px; border-radius: 12px 12px 2px 12px;
    font-size: 12px;
  }
  .ai-msg.assistant .ai-msg-content {
    background: var(--surface-2); padding: 10px 12px; border-radius: 2px 12px 12px 12px;
    font-size: 12px; line-height: 1.5; color: var(--fg);
  }
  .ai-msg.assistant .ai-msg-content :global(strong) { font-weight: 700; }
  .ai-msg.assistant .ai-msg-content :global(code) {
    background: var(--surface-3); padding: 1px 4px; border-radius: 3px; font-size: 11px;
  }
  .ai-msg.assistant .ai-msg-content :global(ul) { margin: 6px 0; padding-left: 16px; }
  .ai-msg.assistant .ai-msg-content :global(li) { margin: 2px 0; }

  .ai-actions { display: flex; flex-direction: column; gap: 6px; margin-top: 8px; }
  .ai-action-card {
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r);
    padding: 8px 10px; display: flex; align-items: center; gap: 8px;
  }
  .ai-action-card.applied { opacity: 0.6; }
  .ai-action-type {
    font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em;
    color: var(--orange); background: var(--orange-bg); padding: 2px 6px; border-radius: 4px;
  }
  .ai-action-label { flex: 1; font-size: 11px; color: var(--fg-2); }
  .ai-action-done { font-size: 10px; color: #34d399; font-weight: 600; }
  .btn-sm { padding: 3px 10px; font-size: 10px; }

  .ai-followups { display: flex; flex-direction: column; gap: 4px; margin-top: 8px; }
  .ai-followup-btn {
    padding: 6px 10px; background: none; border: 1px dashed var(--border-2);
    border-radius: var(--r); font-size: 11px; color: var(--fg-3); cursor: pointer;
    text-align: left; transition: all 120ms;
  }
  .ai-followup-btn:hover { border-color: var(--orange); color: var(--orange); }

  .ai-thinking { display: flex; gap: 4px; padding: 12px 0; }
  .dot {
    width: 6px; height: 6px; border-radius: 50%; background: var(--fg-4);
    animation: blink 1.4s infinite both;
  }
  .dot:nth-child(2) { animation-delay: 0.2s; }
  .dot:nth-child(3) { animation-delay: 0.4s; }
  @keyframes blink {
    0%, 80%, 100% { opacity: 0.3; }
    40% { opacity: 1; }
  }

  .ai-input-area {
    padding: 10px 12px; border-top: 1px solid var(--border);
    display: flex; gap: 8px; align-items: flex-end;
  }
  .ai-input {
    flex: 1; resize: none; background: var(--surface-2); border: 1px solid var(--border);
    border-radius: var(--r); padding: 8px 10px; font-size: 12px; color: var(--fg);
    font-family: inherit; line-height: 1.4;
  }
  .ai-input:focus { border-color: var(--orange); outline: none; }
  .ai-input::placeholder { color: var(--fg-4); }
  .ai-send {
    width: 32px; height: 32px; border-radius: 50%; background: var(--orange); color: #fff;
    border: none; font-size: 14px; font-weight: 700; cursor: pointer;
    display: flex; align-items: center; justify-content: center; flex-shrink: 0;
    transition: opacity 120ms;
  }
  .ai-send:disabled { opacity: 0.4; cursor: not-allowed; }
  .ai-send:not(:disabled):hover { opacity: 0.9; }
</style>
