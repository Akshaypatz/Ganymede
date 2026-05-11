<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { setSetting, createMember, listMembers } from "../api";
  import { currentUser, showOnboarding } from "../stores/app";

  const dispatch = createEventDispatcher<{ complete: void }>();

  let name = "";
  let password = "";
  let confirmPassword = "";
  let lockEnabled = true;
  let saving = false;
  let err = "";

  // Hash password using Web Crypto SHA-256
  async function hashPassword(pw: string): Promise<string> {
    const buf = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(pw));
    return Array.from(new Uint8Array(buf)).map(b => b.toString(16).padStart(2, "0")).join("");
  }

  async function handleContinue() {
    err = "";
    if (!name.trim()) { err = "Please tell us what to call you."; return; }
    if (password.length < 4) { err = "Password must be at least 4 characters."; return; }
    if (password !== confirmPassword) { err = "Passwords do not match."; return; }

    saving = true;
    try {
      const hash = await hashPassword(password);
      await setSetting("user_name", name.trim());
      await setSetting("user_password_hash", hash);
      await setSetting("lock_enabled", lockEnabled ? "true" : "false");
      await setSetting("lock_mode", "sleep"); // lock when screen sleeps by default
      currentUser.set({ name: name.trim() });
      // Ensure the user is always a team member
      try {
        const memberList = await listMembers();
        if (!memberList.some(m => m.name === name.trim())) {
          await createMember({ name: name.trim(), color: "#f97316" });
        }
      } catch {}
      showOnboarding.set(false);
      dispatch("complete");
    } catch (e: any) {
      err = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="onboarding-overlay">
  <div class="onboarding-card">
    <div class="ob-logo-wrap">
      <img src="/billdesk-logo.png" alt="Ganymede" class="ob-logo" />
    </div>
    <div class="ob-brand">Ganymede</div>
    <div class="ob-tagline">Your private mission control.</div>

    <div class="ob-form">
      {#if err}
        <div class="ob-error">{err}</div>
      {/if}

      <div class="ob-field">
        <label class="ob-label">What do people call you?</label>
        <input
          class="ob-input"
          placeholder="Your name or nickname"
          bind:value={name}
          maxlength="60"
          autofocus
        />
      </div>

      <div class="ob-field">
        <label class="ob-label">Set a lock password</label>
        <input
          class="ob-input"
          type="password"
          placeholder="At least 4 characters"
          bind:value={password}
        />
        <div class="ob-hint">Used to unlock when the screen wakes. Touch ID will be offered first on supported Macs.</div>
      </div>

      <div class="ob-field">
        <label class="ob-label">Confirm password</label>
        <input
          class="ob-input"
          type="password"
          placeholder="Re-enter password"
          bind:value={confirmPassword}
          on:keydown={(e) => { if (e.key === "Enter") handleContinue(); }}
        />
      </div>

      <label class="ob-toggle-row">
        <div class="ob-toggle-info">
          <span class="ob-toggle-label">Lock when screen sleeps</span>
          <span class="ob-toggle-sub">Require authentication when you return to the app</span>
        </div>
        <div class="toggle-wrap">
          <input type="checkbox" bind:checked={lockEnabled} class="toggle-input" />
          <span class="toggle-track" class:on={lockEnabled}></span>
        </div>
      </label>

      <button class="ob-btn" on:click={handleContinue} disabled={saving}>
        {saving ? "Setting up…" : "Get Started"}
        {#if !saving}<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/></svg>{/if}
      </button>
    </div>
  </div>
</div>

<style>
  .onboarding-overlay {
    position: fixed; inset: 0;
    background: var(--bg);
    display: flex; align-items: center; justify-content: center;
    z-index: 9999;
  }

  .onboarding-card {
    width: 420px;
    display: flex; flex-direction: column; align-items: center;
    gap: 0;
    animation: fadeUp 400ms cubic-bezier(0.16,1,.3,1);
  }

  @keyframes fadeUp {
    from { opacity: 0; transform: translateY(16px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .ob-logo-wrap {
    width: 64px; height: 64px; border-radius: 18px;
    background: #fff; border: 1px solid var(--border);
    display: flex; align-items: center; justify-content: center;
    padding: 10px; margin-bottom: 16px;
    box-shadow: 0 4px 20px rgba(0,0,0,.2);
  }
  .ob-logo { width: 100%; height: 100%; object-fit: contain; }

  .ob-brand {
    font-size: 28px; font-weight: 800; letter-spacing: -0.04em;
    background: linear-gradient(135deg, #fb923c, #f97316);
    -webkit-background-clip: text; -webkit-text-fill-color: transparent;
    background-clip: text;
    margin-bottom: 4px;
  }

  .ob-tagline {
    font-size: 13px; color: var(--fg-3); margin-bottom: 32px;
  }

  .ob-form {
    width: 100%;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 24px;
    display: flex; flex-direction: column; gap: 18px;
  }

  .ob-error {
    font-size: 12px; color: var(--p0);
    background: rgba(239,68,68,.08); border: 1px solid rgba(239,68,68,.25);
    border-radius: var(--r); padding: 8px 12px;
  }

  .ob-field { display: flex; flex-direction: column; gap: 5px; }
  .ob-label { font-size: 11px; font-weight: 700; color: var(--fg-3); text-transform: uppercase; letter-spacing: 0.05em; }
  .ob-input {
    padding: 9px 12px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r);
    font-size: 13px; color: var(--fg); outline: none;
    transition: border-color 120ms;
  }
  .ob-input:focus { border-color: var(--orange); }
  .ob-hint { font-size: 10px; color: var(--fg-4); line-height: 1.4; margin-top: 2px; }

  .ob-toggle-row {
    display: flex; align-items: center; justify-content: space-between;
    cursor: pointer; padding: 10px 12px;
    background: var(--surface-2); border: 1px solid var(--border);
    border-radius: var(--r); gap: 12px;
  }
  .ob-toggle-info { display: flex; flex-direction: column; gap: 2px; }
  .ob-toggle-label { font-size: 12px; font-weight: 600; color: var(--fg); }
  .ob-toggle-sub { font-size: 10px; color: var(--fg-4); }

  /* Toggle switch */
  .toggle-wrap { position: relative; flex-shrink: 0; }
  .toggle-input { position: absolute; opacity: 0; width: 0; height: 0; }
  .toggle-track {
    display: block; width: 36px; height: 20px; border-radius: 10px;
    background: var(--surface-3); border: 1px solid var(--border);
    position: relative; cursor: pointer; transition: background 180ms;
  }
  .toggle-track::after {
    content: ''; position: absolute; top: 2px; left: 2px;
    width: 14px; height: 14px; border-radius: 7px;
    background: var(--fg-4); transition: transform 180ms, background 180ms;
  }
  .toggle-track.on { background: var(--orange); border-color: var(--orange); }
  .toggle-track.on::after { transform: translateX(16px); background: #fff; }

  .ob-btn {
    display: flex; align-items: center; justify-content: center; gap: 7px;
    padding: 11px 20px; border-radius: var(--r);
    background: linear-gradient(135deg, #fb923c, #f97316);
    color: #fff; font-size: 13px; font-weight: 700;
    border: none; cursor: pointer; transition: opacity 120ms;
    margin-top: 4px;
  }
  .ob-btn:hover { opacity: 0.88; }
  .ob-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
