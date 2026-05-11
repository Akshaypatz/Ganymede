<script lang="ts">
  import { onMount } from "svelte";
  import { getSetting } from "../api";
  import { tryBiometricAuth } from "../api";
  import { isLocked, currentUser } from "../stores/app";

  let password = "";
  let err = "";
  let unlocking = false;
  let biometricAvail = false;
  let userName = "";

  $: initials = userName
    ? userName.trim().split(/\s+/).map(w => w[0]).slice(0, 2).join("").toUpperCase()
    : "?";

  onMount(async () => {
    userName = $currentUser?.name ?? (await getSetting("user_name").catch(() => "")) ?? "";
    biometricAvail = true;
  });

  async function hashPassword(pw: string): Promise<string> {
    const buf = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(pw));
    return Array.from(new Uint8Array(buf)).map(b => b.toString(16).padStart(2, "0")).join("");
  }

  async function unlock() {
    if (!password.trim()) { err = "Enter your password."; return; }
    unlocking = true; err = "";
    try {
      const stored = await getSetting("user_password_hash").catch(() => null);
      if (!stored) { isLocked.set(false); return; }
      const hash = await hashPassword(password);
      if (hash === stored) {
        password = "";
        isLocked.set(false);
      } else {
        err = "Incorrect password.";
        password = "";
      }
    } finally {
      unlocking = false;
    }
  }

  async function useTouchId() {
    unlocking = true; err = "";
    try {
      const result = await tryBiometricAuth();
      if (result === "ok") {
        isLocked.set(false);
      } else if (result === "unavailable") {
        biometricAvail = false;
        err = "Touch ID is not available.";
      } else {
        err = "Authentication failed. Please use your password.";
      }
    } catch {
      err = "Could not start biometric auth. Use your password.";
    } finally {
      unlocking = false;
    }
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === "Enter") unlock();
  }
</script>

<div class="lock-overlay">
  <!-- Stars -->
  <div class="stars-sm" aria-hidden="true"></div>
  <div class="stars-md" aria-hidden="true"></div>
  <div class="stars-lg" aria-hidden="true"></div>

  <!-- Nebula glow behind solar system -->
  <div class="nebula" aria-hidden="true"></div>

  <!-- Solar system scene -->
  <div class="solar-system" aria-hidden="true">
    <!-- Orbit rings + planets -->
    <div class="orbit orbit-1">
      <div class="planet planet-1"></div>
    </div>
    <div class="orbit orbit-2">
      <div class="planet planet-2"></div>
    </div>
    <div class="orbit orbit-3">
      <div class="planet planet-3"></div>
    </div>

    <!-- Sun — Billdesk logo -->
    <div class="sun-glow"></div>
    <div class="sun-avatar">
      <img src="/billdesk-logo.png" alt="" class="sun-logo" />
    </div>
  </div>

  <!-- Lock form -->
  <div class="lock-card" role="dialog" aria-label="Screen locked">
    <div class="lock-name">{userName || "Ganymede"}</div>
    <div class="lock-sub">Screen locked · Enter password to continue</div>

    {#if err}
      <div class="lock-error">{err}</div>
    {/if}

    <input
      class="lock-input"
      type="password"
      placeholder="Password"
      bind:value={password}
      on:keydown={handleKey}
      disabled={unlocking}
      autofocus
    />

    <button class="lock-btn" on:click={unlock} disabled={unlocking}>
      {unlocking ? "Verifying…" : "Unlock"}
    </button>

    {#if biometricAvail}
      <button class="lock-bio-btn" on:click={useTouchId} disabled={unlocking}>
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
          <path d="M12 2C9.243 2 7 4.243 7 7v4a5 5 0 0010 0V7c0-2.757-2.243-5-5-5z"/>
          <path d="M19 10v1a7 7 0 01-14 0v-1"/>
          <line x1="12" y1="19" x2="12" y2="23"/>
          <line x1="8" y1="23" x2="16" y2="23"/>
        </svg>
        Use Touch ID
      </button>
    {/if}
  </div>
</div>

<style>
  /* ─── Overlay ─────────────────────────────────────── */
  .lock-overlay {
    position: fixed; inset: 0;
    background: radial-gradient(ellipse at 50% 35%, #0d1428 0%, #060810 50%, #020408 100%);
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    z-index: 9990;
    animation: fadeIn 300ms ease;
    overflow: hidden;
  }
  @keyframes fadeIn {
    from { opacity: 0; }
    to   { opacity: 1; }
  }

  /* ─── Stars ────────────────────────────────────────── */
  .stars-sm, .stars-md, .stars-lg {
    position: absolute; inset: 0; pointer-events: none;
  }
  .stars-sm::before {
    content: '';
    position: absolute; width: 1px; height: 1px; background: transparent;
    box-shadow:
      42px 123px 0 0 rgba(255,255,255,.7),  187px 56px 0 0 rgba(255,255,255,.5),
      310px 289px 0 0 rgba(255,255,255,.8),  62px 430px 0 0 rgba(255,255,255,.4),
      480px 180px 0 0 rgba(255,255,255,.6),  720px 310px 0 0 rgba(255,255,255,.7),
      850px 90px 0 0 rgba(255,255,255,.5),   145px 670px 0 0 rgba(255,255,255,.6),
      600px 550px 0 0 rgba(255,255,255,.4),  980px 220px 0 0 rgba(255,255,255,.8),
      1200px 480px 0 0 rgba(255,255,255,.5), 1380px 120px 0 0 rgba(255,255,255,.6),
      270px 850px 0 0 rgba(255,255,255,.7),  550px 760px 0 0 rgba(255,255,255,.4),
      1050px 650px 0 0 rgba(255,255,255,.5), 1520px 300px 0 0 rgba(255,255,255,.6),
      90px 940px 0 0 rgba(255,255,255,.5),   1650px 440px 0 0 rgba(255,255,255,.7),
      780px 830px 0 0 rgba(255,255,255,.4),  1300px 720px 0 0 rgba(255,255,255,.6),
      1700px 180px 0 0 rgba(255,255,255,.8), 430px 940px 0 0 rgba(255,255,255,.5),
      1150px 30px 0 0 rgba(255,255,255,.7),  820px 490px 0 0 rgba(255,255,255,.3),
      1450px 810px 0 0 rgba(255,255,255,.6), 200px 380px 0 0 rgba(255,255,255,.5),
      1780px 620px 0 0 rgba(255,255,255,.4), 680px 180px 0 0 rgba(255,255,255,.7),
      1550px 950px 0 0 rgba(255,255,255,.5), 350px 700px 0 0 rgba(255,255,255,.6),
      950px 390px 0 0 rgba(255,255,255,.4),  1100px 860px 0 0 rgba(255,255,255,.7),
      500px 45px 0 0 rgba(255,255,255,.6),   1820px 350px 0 0 rgba(255,255,255,.5),
      760px 680px 0 0 rgba(255,255,255,.8),  1250px 200px 0 0 rgba(255,255,255,.4),
      30px 200px 0 0 rgba(255,255,255,.7),   1600px 720px 0 0 rgba(255,255,255,.6),
      880px 120px 0 0 rgba(255,255,255,.5),  400px 510px 0 0 rgba(255,255,255,.4),
      1400px 540px 0 0 rgba(255,255,255,.7), 130px 800px 0 0 rgba(255,255,255,.5),
      1750px 80px 0 0 rgba(255,255,255,.6),  710px 970px 0 0 rgba(255,255,255,.4),
      1050px 350px 0 0 rgba(255,255,255,.8), 340px 160px 0 0 rgba(255,255,255,.5),
      1650px 880px 0 0 rgba(255,255,255,.6), 580px 620px 0 0 rgba(255,255,255,.3);
    animation: twinkle-slow 6s ease-in-out infinite;
  }
  .stars-md::before {
    content: '';
    position: absolute; width: 1.5px; height: 1.5px; background: transparent;
    box-shadow:
      160px 140px 0 0 rgba(255,255,255,.85), 540px 290px 0 0 rgba(255,255,255,.7),
      900px 410px 0 0 rgba(255,255,255,.9),  1180px 150px 0 0 rgba(255,255,255,.75),
      320px 580px 0 0 rgba(255,255,255,.8),  1450px 650px 0 0 rgba(255,255,255,.65),
      740px 750px 0 0 rgba(255,255,255,.9),  1700px 300px 0 0 rgba(255,255,255,.7),
      240px 900px 0 0 rgba(255,255,255,.75), 1100px 480px 0 0 rgba(255,255,255,.85),
      620px 60px 0 0 rgba(255,255,255,.9),   1580px 820px 0 0 rgba(255,255,255,.7),
      80px 550px 0 0 rgba(255,255,255,.65),  1320px 380px 0 0 rgba(255,255,255,.8),
      470px 820px 0 0 rgba(255,255,255,.7),  1780px 500px 0 0 rgba(255,255,255,.85),
      990px 680px 0 0 rgba(255,255,255,.6),  380px 300px 0 0 rgba(255,255,255,.9),
      1680px 160px 0 0 rgba(255,255,255,.75),820px 940px 0 0 rgba(255,255,255,.7);
    animation: twinkle-slow 9s ease-in-out infinite 2s;
  }
  .stars-lg::before {
    content: '';
    position: absolute; width: 2px; height: 2px; background: transparent;
    box-shadow:
      250px 200px 0 0 rgba(255,255,255,.95), 1050px 120px 0 0 rgba(255,255,255,.85),
      680px 450px 0 0 rgba(255,255,255,.9),  1480px 600px 0 0 rgba(255,255,255,.8),
      120px 750px 0 0 rgba(255,255,255,.95), 1700px 800px 0 0 rgba(255,255,255,.85),
      850px 300px 0 0 rgba(255,255,255,.9),  410px 700px 0 0 rgba(255,255,255,.8),
      1250px 900px 0 0 rgba(255,255,255,.95),1600px 200px 0 0 rgba(255,255,255,.85);
    animation: twinkle-fast 4s ease-in-out infinite 1s;
  }
  @keyframes twinkle-slow {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.35; }
  }
  @keyframes twinkle-fast {
    0%, 100% { opacity: 0.9; }
    30%       { opacity: 0.2; }
    70%       { opacity: 1; }
  }

  /* ─── Nebula ───────────────────────────────────────── */
  .nebula {
    position: absolute;
    top: 50%; left: 50%;
    transform: translate(-50%, -60%);
    width: 600px; height: 400px;
    background: radial-gradient(ellipse at 50% 50%,
      rgba(249,115,22,.06) 0%,
      rgba(139,92,246,.04) 40%,
      transparent 70%);
    pointer-events: none;
    border-radius: 50%;
    filter: blur(30px);
    animation: nebula-breathe 8s ease-in-out infinite;
  }
  @keyframes nebula-breathe {
    0%, 100% { opacity: 0.8; transform: translate(-50%,-60%) scale(1); }
    50%       { opacity: 1;   transform: translate(-50%,-60%) scale(1.1); }
  }

  /* ─── Solar system ─────────────────────────────────── */
  .solar-system {
    position: relative;
    width: 300px; height: 300px;
    margin-bottom: 36px;
    animation: slideDown 400ms cubic-bezier(0.16,1,.3,1);
  }
  @keyframes slideDown {
    from { transform: translateY(-20px); opacity: 0; }
    to   { transform: translateY(0);     opacity: 1; }
  }

  /* Sun */
  .sun-glow {
    position: absolute; top: 50%; left: 50%;
    width: 64px; height: 64px;
    transform: translate(-50%, -50%);
    border-radius: 50%;
    background: radial-gradient(circle, #fbbf24 0%, #f97316 50%, #ea580c 100%);
    animation: sun-pulse 3s ease-in-out infinite;
  }
  @keyframes sun-pulse {
    0%, 100% {
      box-shadow:
        0 0 0 6px rgba(251,191,36,.08),
        0 0 30px 8px rgba(249,115,22,.4),
        0 0 70px 20px rgba(249,115,22,.2),
        0 0 120px 40px rgba(249,115,22,.08);
    }
    50% {
      box-shadow:
        0 0 0 8px rgba(251,191,36,.12),
        0 0 50px 14px rgba(249,115,22,.55),
        0 0 100px 30px rgba(249,115,22,.28),
        0 0 160px 55px rgba(249,115,22,.1);
    }
  }
  .sun-avatar {
    position: absolute; top: 50%; left: 50%;
    width: 64px; height: 64px;
    transform: translate(-50%, -50%);
    border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    background: #fff;
    z-index: 2;
    overflow: hidden;
    padding: 10px;
    box-sizing: border-box;
  }
  .sun-logo {
    width: 100%; height: 100%; object-fit: contain;
    display: block;
  }

  /* Orbit rings */
  .orbit {
    position: absolute; top: 50%; left: 50%;
    border-radius: 50%;
    border: 1px solid rgba(255,255,255,.07);
  }
  .orbit-1 {
    width: 120px; height: 120px;
    transform: translate(-50%,-50%) rotate(45deg);
    animation: orbit-spin 9s linear infinite;
  }
  .orbit-2 {
    width: 190px; height: 190px;
    transform: translate(-50%,-50%) rotate(120deg);
    animation: orbit-spin 15s linear infinite reverse;
  }
  .orbit-3 {
    width: 270px; height: 270px;
    transform: translate(-50%,-50%) rotate(200deg);
    animation: orbit-spin 24s linear infinite;
  }
  @keyframes orbit-spin {
    to { transform: translate(-50%,-50%) rotate(calc(var(--start, 0deg) + 360deg)); }
  }
  /* Simpler orbit keyframes that work without custom property: */
  .orbit-1 { animation: orbit-spin-1 9s linear infinite; }
  .orbit-2 { animation: orbit-spin-2 15s linear infinite; }
  .orbit-3 { animation: orbit-spin-3 24s linear infinite; }
  @keyframes orbit-spin-1 {
    from { transform: translate(-50%,-50%) rotate(45deg); }
    to   { transform: translate(-50%,-50%) rotate(405deg); }
  }
  @keyframes orbit-spin-2 {
    from { transform: translate(-50%,-50%) rotate(120deg); }
    to   { transform: translate(-50%,-50%) rotate(-240deg); }
  }
  @keyframes orbit-spin-3 {
    from { transform: translate(-50%,-50%) rotate(200deg); }
    to   { transform: translate(-50%,-50%) rotate(560deg); }
  }

  /* Planets — positioned at top of orbit ring */
  .planet {
    position: absolute;
    top: 0; left: 50%;
    border-radius: 50%;
  }
  .planet-1 {
    width: 10px; height: 10px;
    transform: translate(-50%, -50%);
    background: radial-gradient(circle at 35% 35%, #fca5a5, #ef4444);
    box-shadow: 0 0 8px 2px rgba(239,68,68,.6);
    animation: counter-spin-1 9s linear infinite;
  }
  .planet-2 {
    width: 12px; height: 12px;
    transform: translate(-50%, -50%);
    background: radial-gradient(circle at 35% 35%, #bfdbfe, #3b82f6);
    box-shadow: 0 0 10px 3px rgba(59,130,246,.5);
    animation: counter-spin-2 15s linear infinite;
  }
  .planet-3 {
    width: 8px; height: 8px;
    transform: translate(-50%, -50%);
    background: radial-gradient(circle at 35% 35%, #fde68a, #d97706);
    box-shadow: 0 0 7px 2px rgba(217,119,6,.5);
    animation: counter-spin-3 24s linear infinite;
  }
  /* Counter-rotation to keep planets upright */
  @keyframes counter-spin-1 {
    from { transform: translate(-50%,-50%) rotate(-45deg); }
    to   { transform: translate(-50%,-50%) rotate(-405deg); }
  }
  @keyframes counter-spin-2 {
    from { transform: translate(-50%,-50%) rotate(-120deg); }
    to   { transform: translate(-50%,-50%) rotate(240deg); }
  }
  @keyframes counter-spin-3 {
    from { transform: translate(-50%,-50%) rotate(-200deg); }
    to   { transform: translate(-50%,-50%) rotate(-560deg); }
  }

  /* ─── Lock card ────────────────────────────────────── */
  .lock-card {
    display: flex; flex-direction: column; align-items: center; gap: 10px;
    width: 300px;
    animation: slideUp 400ms cubic-bezier(0.16,1,.3,1) 100ms both;
  }
  @keyframes slideUp {
    from { transform: translateY(20px); opacity: 0; }
    to   { transform: translateY(0);    opacity: 1; }
  }

  .lock-name {
    font-size: 18px; font-weight: 700; color: rgba(255,255,255,.95);
    letter-spacing: -0.03em;
  }
  .lock-sub {
    font-size: 11px; color: rgba(255,255,255,.35);
    letter-spacing: 0.02em;
    margin-bottom: 4px;
  }
  .lock-error {
    font-size: 11px; color: #fca5a5;
    background: rgba(239,68,68,.1); border: 1px solid rgba(239,68,68,.25);
    border-radius: 8px; padding: 7px 14px; text-align: center; width: 100%;
  }
  .lock-input {
    width: 100%; padding: 10px 14px;
    background: rgba(255,255,255,.05);
    border: 1px solid rgba(255,255,255,.1);
    border-radius: 10px; font-size: 14px; color: #fff; outline: none;
    text-align: center; letter-spacing: 0.12em;
    transition: border-color 150ms, background 150ms;
    box-sizing: border-box;
  }
  .lock-input:focus {
    border-color: rgba(249,115,22,.5);
    background: rgba(255,255,255,.08);
  }
  .lock-input::placeholder { letter-spacing: 0; color: rgba(255,255,255,.25); font-size: 13px; }
  .lock-btn {
    width: 100%; padding: 11px 0; border-radius: 10px;
    background: linear-gradient(135deg, #fb923c 0%, #f97316 60%, #ea580c 100%);
    color: #fff; font-size: 13px; font-weight: 700;
    border: none; cursor: pointer; transition: opacity 120ms, transform 80ms;
    box-shadow: 0 4px 20px rgba(249,115,22,.3);
  }
  .lock-btn:hover  { opacity: 0.9; transform: translateY(-1px); }
  .lock-btn:active { transform: translateY(0); }
  .lock-btn:disabled { opacity: 0.45; cursor: not-allowed; transform: none; }

  .lock-bio-btn {
    display: flex; align-items: center; gap: 7px;
    padding: 8px 20px; border-radius: 8px;
    background: rgba(255,255,255,.04);
    border: 1px solid rgba(255,255,255,.1);
    color: rgba(255,255,255,.55); font-size: 12px; font-weight: 600;
    cursor: pointer; transition: all 150ms; margin-top: 2px;
  }
  .lock-bio-btn:hover { border-color: rgba(249,115,22,.4); color: #fb923c; background: rgba(249,115,22,.06); }
  .lock-bio-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
